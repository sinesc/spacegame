mod inertia;
pub use self::inertia::run as run_inertia;

mod render;
pub use self::render::Render;

mod control;
pub use self::control::run as run_control;

mod compute;
pub use self::compute::run as run_compute;

mod cleanup;
pub use self::cleanup::run as run_cleanup;

mod collider;
pub use self::collider::run as run_collider;

mod upgrader;
pub use self::upgrader::run as run_upgrader;
