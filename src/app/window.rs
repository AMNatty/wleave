mod window_impl {
    use glib::object::ObjectExt;
    use glib::subclass::object::DerivedObjectProperties;
    use glib::subclass::object::{ObjectImpl, ObjectImplExt};
    use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt};
    use glib_macros::Properties;
    use gtk4::prelude::WidgetExt;
    use gtk4::subclass::application_window::ApplicationWindowImpl;
    use gtk4::subclass::widget::{WidgetImpl, WidgetImplExt};
    use gtk4::subclass::window::WindowImpl;
    use libadwaita::subclass::application_window::AdwApplicationWindowImpl;
    use std::cell::Cell;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::WleaveWindow)]
    pub struct WleaveWindowImpl {
        #[property(name = "window-width", get, set)]
        pub window_width: Cell<i32>,
        #[property(name = "window-height", get, set)]
        pub window_height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WleaveWindowImpl {
        const NAME: &'static str = "WleaveWindow";

        type Type = super::WleaveWindow;

        type ParentType = libadwaita::ApplicationWindow;

        type Interfaces = ();
    }

    #[glib::derived_properties]
    impl ObjectImpl for WleaveWindowImpl {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_window_width(obj.width());
            obj.set_window_height(obj.height());
        }
    }

    impl WidgetImpl for WleaveWindowImpl {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);

            let obj = self.obj();
            obj.set_window_width(obj.width());
            obj.set_window_height(obj.height());
        }
    }
    impl WindowImpl for WleaveWindowImpl {}
    impl ApplicationWindowImpl for WleaveWindowImpl {}
    impl AdwApplicationWindowImpl for WleaveWindowImpl {}
}

glib::wrapper! {
    pub struct WleaveWindow(ObjectSubclass<window_impl::WleaveWindowImpl>)
        @extends libadwaita::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gtk4::gio::ActionGroup, gtk4::gio::ActionMap, gtk4::Accessible,
                    gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native, gtk4::Root,
                    gtk4::ShortcutManager;
}

impl WleaveWindow {
    pub fn new(app: &libadwaita::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .property("title", "wleave")
            .property("window-width", 400)
            .property("window-height", 400)
            .build()
    }
}
