pub mod did_accessor;
pub mod sidetree_client;
pub mod studio_client;

pub trait UnwrapLog<T, E> {
    fn unwrap_log(self) -> T;
}

impl<T, E> UnwrapLog<T, E> for Result<T, E>
where
    E: std::fmt::Debug,
{
    fn unwrap_log(self) -> T {
        self.map_err(|e| log::error!("{:?}", e)).unwrap()
    }
}
