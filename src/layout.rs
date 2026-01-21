use crate::layout::menu_layout::LayoutMenuImpl;
use crate::layout::menu_layout_child::MenuLayoutChildImpl;
use glib::object::Cast;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::prelude::{LayoutManagerExt, WidgetExt};
use libadwaita::gtk;
use wleave::cli_opt::MenuLayoutStrategy;

mod menu_layout_child {
    use gdk4::prelude::ObjectExt;
    use glib::subclass::object::{DerivedObjectProperties, ObjectImpl};
    use glib::subclass::types::ObjectSubclass;
    use glib_macros::Properties;
    use gtk4::subclass::layout_child::LayoutChildImpl;
    use std::cell::Cell;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::MenuLayoutChild)]
    pub struct MenuLayoutChildImpl {
        #[property(
            name = "placement",
            get,
            set,
            builder(super::MenuLayoutChildPlacement::default())
        )]
        pub placement: Cell<super::MenuLayoutChildPlacement>,
        #[property(name = "grid-x", get, set)]
        pub grid_x: Cell<u32>,
        #[property(name = "grid-y", get, set)]
        pub grid_y: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MenuLayoutChildImpl {
        const NAME: &'static str = "MenuLayoutChild";

        type Type = super::MenuLayoutChild;

        type ParentType = gtk4::LayoutChild;

        type Interfaces = ();
    }

    #[glib::derived_properties]
    impl ObjectImpl for MenuLayoutChildImpl {}

    impl LayoutChildImpl for MenuLayoutChildImpl {}
}

glib::wrapper! {
    pub struct MenuLayoutChild(ObjectSubclass<MenuLayoutChildImpl>)
        @extends gtk4::LayoutChild;
}

impl MenuLayoutChild {
    fn new(
        manager: &MenuLayout,
        child: &gtk4::Widget,
        placement: MenuLayoutChildPlacement,
    ) -> MenuLayoutChild {
        glib::Object::builder()
            .property("layout-manager", manager)
            .property("child-widget", child)
            .property("placement", placement)
            .build()
    }
}

#[derive(Default, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, glib::Enum)]
#[enum_type(name = "MenuLayoutChildPlacement")]
pub enum MenuLayoutChildPlacement {
    #[default]
    Unknown,
    Grid,
}

mod menu_layout {
    use crate::layout::{MenuLayoutChild, MenuLayoutChildPlacement, MenuLayoutProvider};
    use gdk4::prelude::ObjectExt;
    use gdk4::subclass::prelude::DerivedObjectProperties;
    use glib::object::Cast;
    use glib::subclass::object::ObjectImpl;
    use glib::subclass::prelude::ObjectSubclassExt;
    use glib::subclass::types::ObjectSubclass;
    use glib::types::StaticType;
    use glib_macros::Properties;
    use gtk4::prelude::WidgetExt;
    use gtk4::subclass::layout_manager::LayoutManagerImpl;
    use std::cell::{Cell, RefCell};
    use tracing::instrument;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::MenuLayout)]
    pub struct LayoutMenuImpl {
        #[property(name = "aspect-ratio", get, set)]
        aspect_ratio: Cell<f64>,
        #[property(name = "buttons-per-row", get, set)]
        buttons_per_row: Cell<u32>,
        #[property(name = "aspect-ratio-set", get, set)]
        aspect_ratio_set: Cell<bool>,
        #[property(name = "column-spacing", get, set)]
        column_spacing: Cell<f64>,
        #[property(name = "row-spacing", get, set)]
        row_spacing: Cell<f64>,
        pub(super) layout_strategy: RefCell<MenuLayoutProvider>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LayoutMenuImpl {
        const NAME: &'static str = "MenuLayout";

        type Type = super::MenuLayout;

        type ParentType = gtk4::LayoutManager;

        type Interfaces = ();
    }

    #[glib::derived_properties]
    impl ObjectImpl for LayoutMenuImpl {}

    impl LayoutManagerImpl for LayoutMenuImpl {
        #[instrument(skip(self, widget))]
        fn allocate(&self, widget: &gtk4::Widget, width: i32, height: i32, baseline: i32) {
            {
                let mut layout = self.layout_strategy.borrow_mut();

                layout.column_spacing = self.column_spacing.get();
                layout.row_spacing = self.row_spacing.get();
                layout.aspect_ratio = self.aspect_ratio_set.get().then(|| self.aspect_ratio.get());
            }

            let layout = self.layout_strategy.borrow();

            let mut curr = widget.first_child();
            let children = std::iter::from_fn(|| {
                let it = curr.take()?;
                curr = it.next_sibling();
                Some(it)
            })
            .collect::<Vec<_>>();

            layout.allocate(&self.obj(), &children, width, height, baseline);
        }

        fn create_layout_child(
            &self,
            _widget: &gtk4::Widget,
            for_child: &gtk4::Widget,
        ) -> gtk4::LayoutChild {
            MenuLayoutChild::new(
                &self.obj(),
                for_child,
                match self.layout_strategy.borrow().strategy {
                    super::MenuLayoutStrategy::Grid => MenuLayoutChildPlacement::Grid,
                },
            )
            .upcast()
        }

        fn layout_child_type() -> Option<glib::Type> {
            Some(MenuLayoutChild::static_type())
        }

        fn request_mode(&self, _widget: &gtk4::Widget) -> gtk4::SizeRequestMode {
            gtk4::SizeRequestMode::HeightForWidth
        }

        #[instrument(skip(self, widget))]
        fn measure(
            &self,
            widget: &gtk4::Widget,
            orientation: gtk4::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let mut curr = widget.first_child();
            let children = std::iter::from_fn(|| {
                let it = curr.take()?;
                curr = it.next_sibling();
                Some(it)
            })
            .collect::<Vec<_>>();

            let layout = self.layout_strategy.borrow();

            layout.measure(&self.obj(), &children, orientation, for_size)
        }
    }
}

glib::wrapper! {
    pub struct MenuLayout(ObjectSubclass<LayoutMenuImpl>)
        @extends gtk4::LayoutManager;
}

impl MenuLayout {
    pub fn new(
        button_layout: MenuLayoutStrategy,
        ratio: Option<impl Into<f64>>,
        column_spacing: gtk4::Expression,
        row_spacing: gtk4::Expression,
        buttons_per_row: Option<impl Into<u32>>,
    ) -> Self {
        let obj: MenuLayout = glib::Object::builder()
            .property("aspect-ratio-set", ratio.is_some())
            .property("aspect-ratio", ratio.map(Into::into).unwrap_or(1.0))
            .property(
                "buttons-per-row",
                buttons_per_row.map(Into::into).unwrap_or_default(),
            )
            .build();

        column_spacing.bind(&obj, "column-spacing", glib::Object::NONE);
        row_spacing.bind(&obj, "row-spacing", glib::Object::NONE);

        let imp = obj.imp();
        imp.layout_strategy.borrow_mut().strategy = button_layout;

        obj
    }
}

#[derive(Default)]
struct MenuLayoutProvider {
    strategy: MenuLayoutStrategy,
    column_spacing: f64,
    row_spacing: f64,
    aspect_ratio: Option<f64>,
}

impl MenuLayoutProvider {
    fn allocate(
        &self,
        obj: &MenuLayout,
        children: &[gtk4::Widget],
        width: i32,
        height: i32,
        baseline: i32,
    ) {
        if children.is_empty() {
            return;
        }

        match self.strategy {
            MenuLayoutStrategy::Grid => {
                let n = children.len();
                let col_spacing = (self.column_spacing as i32).max(0) as usize;
                let row_spacing = (self.row_spacing as i32).max(0) as usize;

                let mut rows = 1;
                let mut cols = 1;
                let mut b_width = 0.0;
                let mut b_height = 0.0;

                let u_width = width as usize;
                let u_height = height as usize;

                let per_row = obj.buttons_per_row() as usize;
                // We use 0 for "auto" placement
                let cols_range = if per_row != 0 {
                    // Try layouts where all buttons either fit into one row or exactly "per_row"
                    per_row.min(n)..=per_row
                } else {
                    // Try all possible layouts
                    1..=n
                };

                // Axis-aligned rectangle packing
                // We brute-force the best layout, optimizing for max button area
                for i_rows in 1..=n {
                    for j_cols in cols_range.clone() {
                        if (i_rows * j_cols > n + i_rows || i_rows * j_cols > n + j_cols)
                            && per_row == 0
                            || i_rows * j_cols < n
                        {
                            continue;
                        }

                        let col_gaps = j_cols - 1;
                        let row_gaps = i_rows - 1;

                        let (w, h) = match self.aspect_ratio {
                            Some(aspect @ 1.0..) => {
                                let mut w = u_width.saturating_sub(col_gaps * col_spacing) as f64
                                    / j_cols as f64
                                    * aspect;
                                let h = (u_height.saturating_sub(row_gaps * row_spacing) as f64
                                    / i_rows as f64)
                                    .min(w / aspect);

                                w = h * aspect;

                                (w, h)
                            }
                            Some(aspect @ ..1.0) => {
                                let mut h = u_height.saturating_sub(row_gaps * row_spacing) as f64
                                    / i_rows as f64;
                                let w = (u_width.saturating_sub(col_gaps * col_spacing) as f64
                                    / j_cols as f64
                                    * aspect)
                                    .min(h * aspect);

                                h = w / aspect;

                                (w, h)
                            }
                            //
                            Some(..) | None => {
                                let w = u_width.saturating_sub(col_gaps * col_spacing) as f64
                                    / j_cols as f64;
                                let h = u_height.saturating_sub(row_gaps * row_spacing) as f64
                                    / i_rows as f64;

                                (w, h)
                            }
                        };

                        if w * h > b_width * b_height {
                            rows = i_rows;
                            cols = j_cols;
                            b_width = w;
                            b_height = h;
                        }
                    }
                }

                let base_x =
                    (width as f64 - (cols - 1) as f64 * (col_spacing as f64 + b_width) - b_width)
                        / 2.0;
                let base_y = (height as f64
                    - (rows - 1) as f64 * (row_spacing as f64 + b_height)
                    - b_height)
                    / 2.0;

                for (i, child) in children.iter().enumerate() {
                    let child_layout = obj
                        .layout_child(child)
                        .downcast::<MenuLayoutChild>()
                        .expect("always MenuLayoutChild");

                    if child.should_layout() {
                        let ix = i % cols;
                        let iy = i / cols;

                        child_layout.set_placement(MenuLayoutChildPlacement::Grid);
                        child_layout.set_grid_x(ix as u32);
                        child_layout.set_grid_y(iy as u32);

                        let x_grid = ix as f64;
                        let y_grid = iy as f64;

                        let x = base_x
                            + x_grid * b_width
                            + x_grid * self.column_spacing * self.aspect_ratio.unwrap_or(1.0);
                        let y = base_y + y_grid * b_height + y_grid * self.row_spacing;

                        child.size_allocate(
                            &gtk4::Allocation::new(
                                x as i32,
                                y as i32,
                                b_width as i32,
                                b_height as i32,
                            ),
                            baseline,
                        );
                    }
                }
            }
        }
    }

    fn measure(
        &self,
        _obj: &MenuLayout,
        children: &[gtk4::Widget],
        orientation: gtk4::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        let gaps = match orientation {
            gtk::Orientation::Vertical => self.row_spacing as i32,
            gtk::Orientation::Horizontal => self.column_spacing as i32,
            _ => 0,
        };

        let (mut min, mut nat) = (0, 0);

        match self.strategy {
            MenuLayoutStrategy::Grid => {
                for child in children.iter() {
                    if !child.should_layout() {
                        continue;
                    }

                    let (c_min, c_nat, _, _) = child.measure(orientation, for_size);

                    min = min.max(c_min + gaps);
                    nat = nat.max(c_nat + gaps);
                }

                // A bit of a messy heuristic
                if matches!(orientation, gtk::Orientation::Horizontal) {
                    min *= children.len() as i32;
                    nat *= children.len() as i32;
                } else {
                    min *= children.len() as i32;
                    min /= 2;
                    nat *= children.len() as i32;
                    nat /= 2;
                }
            }
        }

        (min, nat, -1, -1)
    }
}
