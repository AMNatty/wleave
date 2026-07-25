use crate::layout::menu_layout::LayoutMenuImpl;
use crate::layout::menu_layout_child::MenuLayoutChildImpl;
use glam::{DVec2, USizeVec2};
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
                let spacing = USizeVec2::new(
                    (self.column_spacing as i32).max(0) as usize,
                    (self.row_spacing as i32).max(0) as usize,
                );

                let available_box = glam::USizeVec2::new(width as usize, height as usize);

                let per_row = obj.buttons_per_row() as usize;

                let mut best_grid = glam::USizeVec2::new(n, 1);
                let mut best_box = glam::USizeVec2::ZERO;
                let epsilon = 1e-2;
                let bounds = available_box.as_dvec2();
                let spacing_d = spacing.as_dvec2();
                let aspect_fac = self
                    .aspect_ratio
                    .map(|aspect| glam::DVec2::new(aspect, 1.0));

                match (aspect_fac, per_row) {
                    (Some(aspect), 0) => {
                        let mut best_height = 0.0;

                        for cols in 1..=n {
                            let cells_u = USizeVec2::new(cols, n.div_ceil(cols));
                            let cells = cells_u.as_dvec2();
                            let space_per_box = (bounds - (cells - DVec2::ONE) * spacing_d) / cells;
                            let box_height = (space_per_box / aspect).x.min(space_per_box.y);

                            if box_height > best_height + epsilon {
                                best_height = box_height;
                                best_grid = cells_u;
                                best_box = (glam::DVec2::splat(box_height) * aspect).as_usizevec2();
                            }
                        }
                    }
                    (None, 0) => {
                        let mut max_area = 0.0;

                        for cols in 1..=n {
                            let cells_u = USizeVec2::new(cols, n.div_ceil(cols));
                            let cells = cells_u.as_dvec2();
                            let space_per_box = (bounds - (cells - DVec2::ONE) * spacing_d) / cells;
                            let area = space_per_box.element_product();

                            if area > max_area + epsilon {
                                max_area = area;
                                best_grid = cells_u;
                                best_box = space_per_box.as_usizevec2();
                            }
                        }
                    }
                    (Some(aspect), per_row) => {
                        best_grid = glam::USizeVec2::new(per_row.min(n), n.div_ceil(per_row));
                        let cells = best_grid.as_dvec2();
                        let space_per_box = (bounds - (cells - DVec2::ONE) * spacing_d) / cells;
                        let box_height = (space_per_box / aspect).x.min(space_per_box.y);
                        best_box = (glam::DVec2::splat(box_height) * aspect).as_usizevec2();
                    }
                    (None, per_row) => {
                        best_grid = glam::USizeVec2::new(per_row.min(n), n.div_ceil(per_row));
                        best_box =
                            (available_box - (best_grid - USizeVec2::ONE) * spacing) / best_grid;
                    }
                }

                let used_box = best_grid * (spacing + best_box) - spacing;
                let base_point = (available_box - used_box) / 2;

                for (i, child) in children.iter().enumerate() {
                    let child_layout = obj
                        .layout_child(child)
                        .downcast::<MenuLayoutChild>()
                        .expect("always MenuLayoutChild");

                    if child.should_layout() {
                        let grid_pos = USizeVec2::new(i % best_grid.x, i / best_grid.x);

                        child_layout.set_placement(MenuLayoutChildPlacement::Grid);
                        child_layout.set_grid_x(grid_pos.x as u32);
                        child_layout.set_grid_y(grid_pos.y as u32);

                        let pos = base_point + grid_pos * (best_box + spacing);

                        child.size_allocate(
                            &gtk4::Allocation::new(
                                pos.x as i32,
                                pos.y as i32,
                                best_box.x as i32,
                                best_box.y as i32,
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
