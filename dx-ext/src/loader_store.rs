use dioxus::{
    CapturedError,
    core::{SuperInto, SuspendedFuture},
    fullstack::LoaderState,
    prelude::*,
};
use std::future::Future;

/// A Loader is a signal that represents a value that is loaded asynchronously.
///
/// Once a `Loader<T>` has been successfully created from `use_loader`, it can be use like a normal signal of type `T`.
///
/// When the loader is re-reloading its values, it will no longer suspend its component, making it
/// very useful for server-side-rendering.
pub struct LoaderStore<T: 'static> {
    /// This is a signal that unwraps the inner value. We can't give it out unless we know the inner value is Some(T)!
    read_value: ReadStore<T>,

    /// This is the actual signal. We let the user set this value if they want to, but we can't let them set it to `None`.
    real_value: Store<Option<T>>,
    error: Signal<Option<CapturedError>>,
    state: Signal<LoaderState>,
    handle: LoaderHandle,
}

#[track_caller]
#[allow(clippy::result_large_err)]
pub fn use_loader_store<F, T, E>(
    mut future: impl FnMut() -> F + 'static,
) -> Result<LoaderStore<T>, Loading>
where
    F: Future<Output = Result<T, E>> + 'static,
    T: 'static + PartialEq + serde::Serialize + serde::de::DeserializeOwned,
    E: Into<CapturedError> + 'static,
{
    let serialize_context = use_hook(dioxus::fullstack::serialize_context);

    // We always create a storage entry, even if the data isn't ready yet to make it possible to deserialize pending server futures on the client
    #[allow(unused)]
    let storage_entry: dioxus::fullstack::SerializeContextEntry<Result<T, CapturedError>> =
        use_hook(|| serialize_context.create_entry());

    #[cfg(feature = "server")]
    let caller = std::panic::Location::caller();

    // If this is the first run and we are on the web client, the data might be cached
    #[cfg(feature = "web")]
    let initial_web_result =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(Some(storage_entry.get()))));

    let mut error = use_signal(|| None as Option<CapturedError>);
    let mut value = use_store(|| None as Option<T>);
    let mut loader_state = use_signal(|| LoaderState::Pending);

    let resource = use_resource(move || {
        #[cfg(feature = "server")]
        let storage_entry = storage_entry.clone();

        let user_fut = future();

        #[cfg(feature = "web")]
        let initial_web_result = initial_web_result.clone();

        #[allow(clippy::let_and_return)]
        async move {
            // If this is the first run and we are on the web client, the data might be cached
            #[cfg(feature = "web")]
            match initial_web_result.take() {
                // The data was deserialized successfully from the server
                Some(Ok(o)) => {
                    match o {
                        Ok(v) => {
                            value.set(Some(v));
                            loader_state.set(LoaderState::Ready);
                        }
                        Err(e) => {
                            error.set(Some(e));
                            loader_state.set(LoaderState::Failed);
                        }
                    };
                    return;
                }

                // The data is still pending from the server. Don't try to resolve it on the client
                Some(Err(dioxus::fullstack::TakeDataError::DataPending)) => {
                    std::future::pending::<()>().await
                }

                // The data was not available on the server, rerun the future
                Some(Err(_)) => {}

                // This isn't the first run, so we don't need do anything
                None => {}
            }

            // Otherwise just run the future itself
            let out = user_fut.await;

            // Remap the error to the captured error type so it's cheap to clone and pass out, just
            // slightly more cumbersome to access the inner error.
            let out = out.map_err(|e| {
                let anyhow_err: CapturedError = e.into();
                anyhow_err
            });

            // If this is the first run and we are on the server, cache the data in the slot we reserved for it
            #[cfg(feature = "server")]
            storage_entry.insert(&out, caller);

            match out {
                Ok(v) => {
                    value.set(Some(v));
                    loader_state.set(LoaderState::Ready);
                }
                Err(e) => {
                    error.set(Some(e));
                    loader_state.set(LoaderState::Failed);
                }
            };
        }
    });

    // On the first run, force this task to be polled right away in case its value is ready
    use_hook(|| {
        let _ = resource.task().poll_now();
    });

    let read_value: ReadStore<T> = use_hook(|| {
        value
            .selector()
            .map(|v: &Option<T>| v.as_ref().unwrap(), |v| v.as_mut().unwrap())
    })
    .map_writer(std::convert::Into::into)
    .into();

    let handle = LoaderHandle {
        resource,
        error,
        state: loader_state,
        _marker: std::marker::PhantomData,
    };

    match &*loader_state.read_unchecked() {
        LoaderState::Pending => Err(Loading::Pending(handle)),
        LoaderState::Failed => Err(Loading::Failed(handle)),
        LoaderState::Ready => Ok(LoaderStore {
            real_value: value,
            read_value,
            error,
            state: loader_state,
            handle,
        }),
    }
}

#[track_caller]
#[allow(clippy::result_large_err)]
pub fn use_mapped_loader_store<AsyncF, Inter, T, E>(
    mut future: impl FnMut() -> AsyncF + 'static,
    map: impl Fn(Inter) -> T + Clone + 'static,
) -> Result<LoaderStore<T>, Loading>
where
    AsyncF: Future<Output = Result<Inter, E>> + 'static,
    Inter: 'static + PartialEq + serde::Serialize + serde::de::DeserializeOwned,
    E: Into<CapturedError> + 'static,
{
    let serialize_context = use_hook(dioxus::fullstack::serialize_context);

    // We always create a storage entry, even if the data isn't ready yet to make it possible to deserialize pending server futures on the client
    #[allow(unused)]
    let storage_entry: dioxus::fullstack::SerializeContextEntry<Result<Inter, CapturedError>> =
        use_hook(|| serialize_context.create_entry());

    #[cfg(feature = "server")]
    let caller = std::panic::Location::caller();

    // If this is the first run and we are on the web client, the data might be cached
    #[cfg(feature = "web")]
    let initial_web_result =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(Some(storage_entry.get()))));

    let mut error = use_signal(|| None as Option<CapturedError>);
    let mut value = use_store(|| None as Option<T>);
    let mut loader_state = use_signal(|| LoaderState::Pending);

    let resource = use_resource(move || {
        #[cfg(feature = "server")]
        let storage_entry = storage_entry.clone();

        let user_fut = future();

        #[cfg(feature = "web")]
        let initial_web_result = initial_web_result.clone();

        let map = map.clone();

        #[allow(clippy::let_and_return)]
        async move {
            // If this is the first run and we are on the web client, the data might be cached
            #[cfg(feature = "web")]
            match initial_web_result.take() {
                // The data was deserialized successfully from the server
                Some(Ok(o)) => {
                    match o {
                        Ok(v) => {
                            value.set(Some(v));
                            loader_state.set(LoaderState::Ready);
                        }
                        Err(e) => {
                            error.set(Some(e));
                            loader_state.set(LoaderState::Failed);
                        }
                    };
                    return;
                }

                // The data is still pending from the server. Don't try to resolve it on the client
                Some(Err(dioxus::fullstack::TakeDataError::DataPending)) => {
                    std::future::pending::<()>().await
                }

                // The data was not available on the server, rerun the future
                Some(Err(_)) => {}

                // This isn't the first run, so we don't need do anything
                None => {}
            }

            // Otherwise just run the future itself
            let out = user_fut.await;

            // Remap the error to the captured error type so it's cheap to clone and pass out, just
            // slightly more cumbersome to access the inner error.
            let out = out.map_err(|e| {
                let anyhow_err: CapturedError = e.into();
                anyhow_err
            });

            // If this is the first run and we are on the server, cache the data in the slot we reserved for it
            #[cfg(feature = "server")]
            storage_entry.insert(&out, caller);

            match out {
                Ok(v) => {
                    value.set(Some(map(v)));
                    loader_state.set(LoaderState::Ready);
                }
                Err(e) => {
                    error.set(Some(e));
                    loader_state.set(LoaderState::Failed);
                }
            };
        }
    });

    // On the first run, force this task to be polled right away in case its value is ready
    use_hook(|| {
        let _ = resource.task().poll_now();
    });

    let read_value: ReadStore<T> = use_hook(|| {
        value
            .selector()
            .map(|v: &Option<T>| v.as_ref().unwrap(), |v| v.as_mut().unwrap())
    })
    .map_writer(std::convert::Into::into)
    .into();

    let handle = LoaderHandle {
        resource,
        error,
        state: loader_state,
        _marker: std::marker::PhantomData,
    };

    match &*loader_state.read_unchecked() {
        LoaderState::Pending => Err(Loading::Pending(handle)),
        LoaderState::Failed => Err(Loading::Failed(handle)),
        LoaderState::Ready => Ok(LoaderStore {
            real_value: value,
            read_value,
            error,
            state: loader_state,
            handle,
        }),
    }
}

impl<T: 'static> LoaderStore<T> {
    /// Get the error that occurred during loading, if any.
    ///
    /// After initial load, this will return `None` until the next reload fails.
    pub fn error(&self) -> Option<CapturedError> {
        self.error.read().as_ref().cloned()
    }

    /// Restart the loading task.
    ///
    /// After initial load, this won't suspend the component, but will reload in the background.
    pub fn restart(&mut self) {
        self.handle.restart();
    }

    /// Check if the loader has failed.
    pub fn is_error(&self) -> bool {
        self.error.read().is_some() && matches!(*self.state.read(), LoaderState::Failed)
    }

    /// Cancel the current loading task.
    pub fn cancel(&mut self) {
        self.handle.resource.cancel();
    }

    pub fn loading(&self) -> bool {
        !self.handle.resource.finished()
    }
}

impl Copy for LoaderHandle {}

impl<T: 'static> Writable for LoaderStore<T> {
    type WriteMetadata = <Store<T> as Writable>::WriteMetadata;

    fn try_write_unchecked(
        &self,
    ) -> std::result::Result<dioxus_signals::WritableRef<'static, Self>, BorrowMutError>
    where
        Self::Target: 'static,
    {
        let writer = self.real_value.try_write_unchecked()?;
        Ok(WriteLock::map(writer, |v| {
            v.as_mut()
                .expect("LoaderStore should never be None when writing")
        }))
    }
}

impl<T> Readable for LoaderStore<T> {
    type Target = T;
    type Storage = UnsyncStorage;

    #[track_caller]
    fn try_read_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError>
    where
        T: 'static,
    {
        Ok(self.read_value.read_unchecked())
    }

    /// Get the current value of the signal. **Unlike read, this will not subscribe the current scope to the signal which can cause parts of your UI to not update.**
    ///
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    fn try_peek_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError>
    where
        T: 'static,
    {
        Ok(self.read_value.peek_unchecked())
    }

    fn subscribers(&self) -> dioxus::core::Subscribers
    where
        T: 'static,
    {
        self.read_value.subscribers()
    }
}

impl<T> dioxus::core::IntoAttributeValue for LoaderStore<T>
where
    T: Clone + dioxus::core::IntoAttributeValue + PartialEq + 'static,
{
    fn into_value(self) -> dioxus::core::AttributeValue {
        self.with(|f| f.clone().into_value())
    }
}

impl<T> dioxus::core::IntoDynNode for LoaderStore<T>
where
    T: Clone + dioxus::core::IntoDynNode + PartialEq + 'static,
{
    fn into_dyn_node(self) -> dioxus_core::DynamicNode {
        let t: T = self();
        t.into_dyn_node()
    }
}

impl<T: 'static> PartialEq for LoaderStore<T> {
    fn eq(&self, other: &Self) -> bool {
        self.read_value == other.read_value
    }
}

impl<T: Clone> std::ops::Deref for LoaderStore<T>
where
    T: PartialEq + 'static,
{
    type Target = dyn Fn() -> T;

    fn deref(&self) -> &Self::Target {
        unsafe { ReadableExt::deref_impl(self) }
    }
}

read_impls!(LoaderStore<T> where T: PartialEq);

impl<T> Clone for LoaderStore<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for LoaderStore<T> {}

impl<T> std::convert::From<LoaderStore<T>> for ReadStore<T> {
    fn from(loader: LoaderStore<T>) -> Self {
        loader.read_value
    }
}

impl<T> std::convert::From<LoaderStore<T>> for Store<Option<T>> {
    fn from(loader: LoaderStore<T>) -> Self {
        loader.real_value
    }
}

impl<T> dioxus::core::SuperFrom<LoaderStore<T>> for ReadSignal<T> {
    fn super_from(loader: LoaderStore<T>) -> Self {
        loader.read_value.super_into()
    }
}

impl<T> dioxus::core::SuperFrom<LoaderStore<T>> for WriteSignal<Option<T>> {
    fn super_from(loader: LoaderStore<T>) -> Self {
        loader.real_value.super_into()
    }
}

impl<T> LoaderStore<T> {
    pub fn read_store(&self) -> ReadStore<T> {
        self.read_value
    }

    pub fn store(&self) -> Store<Option<T>> {
        self.real_value
    }
}

#[derive(PartialEq)]
pub struct LoaderHandle<M = ()> {
    pub(crate) resource: Resource<()>,
    pub(crate) error: Signal<Option<CapturedError>>,
    pub(crate) state: Signal<LoaderState>,
    pub(crate) _marker: std::marker::PhantomData<M>,
}

impl LoaderHandle {
    /// Restart the loading task.
    pub fn restart(&mut self) {
        self.resource.restart();
    }

    /// Get the current state of the loader.
    pub fn state(&self) -> LoaderState {
        *self.state.read()
    }

    pub fn error(&self) -> Option<CapturedError> {
        self.error.read().as_ref().cloned()
    }
}

impl Clone for LoaderHandle {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(PartialEq)]
pub enum Loading {
    /// The loader is still pending and the component should suspend.
    Pending(LoaderHandle),

    /// The loader has failed and an error will be returned up the tree.
    Failed(LoaderHandle),
}

impl std::fmt::Debug for Loading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loading::Pending(_) => write!(f, "Loading::Pending"),
            Loading::Failed(_) => write!(f, "Loading::Failed"),
        }
    }
}

impl std::fmt::Display for Loading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loading::Pending(_) => write!(f, "Loading is still pending"),
            Loading::Failed(_) => write!(f, "Loading has failed"),
        }
    }
}

/// Convert a Loading into a RenderError for use with the `?` operator in components
impl From<Loading> for RenderError {
    fn from(val: Loading) -> Self {
        match val {
            Loading::Pending(t) => RenderError::Suspended(SuspendedFuture::new(t.resource.task())),
            Loading::Failed(err) => RenderError::Error(
                err.error
                    .cloned()
                    .expect("LoaderHandle in Failed state should always have an error"),
            ),
        }
    }
}
