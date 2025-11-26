pub mod get;
pub mod head;
pub mod put;
pub mod delete;

pub use get::get_object;
pub use head::head_object;
pub use put::put_object;
pub use delete::delete_object;