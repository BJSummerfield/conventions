macro_rules! dispatch {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident: $port:path {
            $( $(#[$var_meta:meta])* $ctor:ident => $variant:ident($adapter:ty) ),+ $(,)?
        }
        $($methods:tt)*
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $( $(#[$var_meta])* $variant($adapter), )+
        }

        impl $name {
            $(
                $(#[$var_meta])*
                pub fn $ctor(inner: $adapter) -> Self {
                    Self::$variant(inner)
                }
            )+
        }

        impl $port for $name {
            $crate::dispatch::dispatch!(
                @methods [$( { $(#[$var_meta])* $variant } )+] $($methods)*
            );
        }
    };

    (@methods $variants:tt) => {};

    (
        @methods $variants:tt
        async fn $method:ident(&self $(, $arg:ident: $ty:ty)* $(,)?) $(-> $ret:ty)?;
        $($rest:tt)*
    ) => {
        $crate::dispatch::dispatch!(
            @async_method $variants $method [$($arg: $ty),*] ($($arg),*) [$($ret)?]
        );
        $crate::dispatch::dispatch!(@methods $variants $($rest)*);
    };

    (
        @methods $variants:tt
        fn $method:ident(&self $(, $arg:ident: $ty:ty)* $(,)?) $(-> $ret:ty)?;
        $($rest:tt)*
    ) => {
        $crate::dispatch::dispatch!(
            @sync_method $variants $method [$($arg: $ty),*] ($($arg),*) [$($ret)?]
        );
        $crate::dispatch::dispatch!(@methods $variants $($rest)*);
    };

    (
        @async_method [$( { $(#[$var_meta:meta])* $variant:ident } )+]
        $method:ident [$($arg:ident: $ty:ty),*] $call:tt [$($ret:ty)?]
    ) => {
        async fn $method(&self, $($arg: $ty),*) $(-> $ret)? {
            match self {
                $( $(#[$var_meta])* Self::$variant(inner) => inner.$method $call.await, )+
            }
        }
    };

    (
        @sync_method [$( { $(#[$var_meta:meta])* $variant:ident } )+]
        $method:ident [$($arg:ident: $ty:ty),*] $call:tt [$($ret:ty)?]
    ) => {
        fn $method(&self, $($arg: $ty),*) $(-> $ret)? {
            match self {
                $( $(#[$var_meta])* Self::$variant(inner) => inner.$method $call, )+
            }
        }
    };
}

pub(crate) use dispatch;

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Mutex;
    use std::task::{Context, Poll, Waker};

    use super::dispatch;

    fn block_on<F: Future>(future: F) -> F::Output {
        let mut future = pin!(future);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
                return output;
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct WidgetName(String);

    #[derive(Debug, PartialEq, Eq)]
    pub(crate) enum WidgetError {
        Repository(String),
    }

    pub(crate) trait WidgetRepository {
        async fn fetch(&self, name: &WidgetName) -> Result<Option<WidgetName>, WidgetError>;
        async fn save(&self, name: WidgetName) -> Result<(), WidgetError>;
        fn backend(&self) -> &'static str;
    }

    #[derive(Debug, Default)]
    pub(crate) struct InMemoryWidgetRepository(Mutex<Vec<WidgetName>>);

    impl WidgetRepository for InMemoryWidgetRepository {
        async fn fetch(&self, name: &WidgetName) -> Result<Option<WidgetName>, WidgetError> {
            let stored = self.0.lock().map_err(|e| WidgetError::Repository(e.to_string()))?;
            Ok(stored.iter().find(|w| *w == name).cloned())
        }

        async fn save(&self, name: WidgetName) -> Result<(), WidgetError> {
            let mut stored = self.0.lock().map_err(|e| WidgetError::Repository(e.to_string()))?;
            stored.push(name);
            Ok(())
        }

        fn backend(&self) -> &'static str {
            "in_memory"
        }
    }

    #[derive(Debug)]
    pub(crate) struct FailingWidgetRepository;

    impl WidgetRepository for FailingWidgetRepository {
        async fn fetch(&self, _name: &WidgetName) -> Result<Option<WidgetName>, WidgetError> {
            Err(WidgetError::Repository("down".into()))
        }

        async fn save(&self, _name: WidgetName) -> Result<(), WidgetError> {
            Err(WidgetError::Repository("down".into()))
        }

        fn backend(&self) -> &'static str {
            "failing"
        }
    }

    dispatch! {
        #[derive(Debug)]
        pub(crate) enum WidgetRepo: WidgetRepository {
            in_memory => InMemory(InMemoryWidgetRepository),
            #[cfg(test)]
            failing => Failing(FailingWidgetRepository),
        }
        async fn fetch(&self, name: &WidgetName) -> Result<Option<WidgetName>, WidgetError>;
        async fn save(&self, name: WidgetName) -> Result<(), WidgetError>;
        fn backend(&self) -> &'static str;
    }

    fn widget(name: &str) -> WidgetName {
        WidgetName(name.to_string())
    }

    mod in_memory {
        use super::*;

        #[test]
        fn delegates_async_methods() {
            let repo = WidgetRepo::in_memory(InMemoryWidgetRepository::default());
            block_on(repo.save(widget("gear"))).unwrap();
            assert_eq!(block_on(repo.fetch(&widget("gear"))), Ok(Some(widget("gear"))));
            assert_eq!(block_on(repo.fetch(&widget("cog"))), Ok(None));
        }

        #[test]
        fn delegates_sync_methods() {
            let repo = WidgetRepo::in_memory(InMemoryWidgetRepository::default());
            assert_eq!(repo.backend(), "in_memory");
        }
    }

    mod failing {
        use super::*;

        #[test]
        fn delegates_errors_unchanged() {
            let repo = WidgetRepo::failing(FailingWidgetRepository);
            assert_eq!(
                block_on(repo.save(widget("gear"))),
                Err(WidgetError::Repository("down".into()))
            );
            assert_eq!(repo.backend(), "failing");
        }
    }
}
