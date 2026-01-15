use gdk4::prelude::ObjectExt;
use gdk4::subclass::prelude::DerivedObjectProperties;
use glib::subclass::object::ObjectImpl;
use glib::subclass::types::ObjectSubclass;
use glib_macros::Properties;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::layout_manager::LayoutManagerImpl;
use std::cell::{Cell, RefCell};
use tracing::instrument;

#[derive(Properties, Default)]
#[properties(wrapper_type = LayoutWleaveMenu)]
pub struct LayoutWleaveMenuImpl {
    #[property(name = "aspect-ratio", get, set)]
    aspect_ratio: Cell<f64>,
    #[property(name = "aspect-ratio-set", get, set)]
    aspect_ratio_set: Cell<bool>,
    #[property(name = "column-spacing", get, set)]
    column_spacing: Cell<f64>,
    #[property(name = "row-spacing", get, set)]
    row_spacing: Cell<f64>,
    layout_strategy: RefCell<MenuLayout>,
}

#[glib::object_subclass]
impl ObjectSubclass for LayoutWleaveMenuImpl {
    const NAME: &'static str = "WleaveLayoutAspect";

    type Type = LayoutWleaveMenu;

    type ParentType = gtk4::LayoutManager;

    type Interfaces = ();
}

glib::wrapper! {
    pub struct LayoutWleaveMenu(ObjectSubclass<LayoutWleaveMenuImpl>)
        @extends gtk4::LayoutManager;
}

#[glib::derived_properties]
impl ObjectImpl for LayoutWleaveMenuImpl {}

impl LayoutManagerImpl for LayoutWleaveMenuImpl {
    #[instrument(skip(self, widget))]
    fn allocate(&self, widget: &gtk4::Widget, width: i32, height: i32, baseline: i32) {
        let mut layout = self.layout_strategy.borrow_mut();

        layout.column_spacing = self.column_spacing.get();
        layout.row_spacing = self.row_spacing.get();
        layout.aspect_ratio = self.aspect_ratio_set.get().then(|| self.aspect_ratio.get());

        let mut curr = widget.first_child();
        let children = std::iter::from_fn(|| {
            let it = curr.take()?;
            curr = it.next_sibling();
            Some(it)
        })
        .collect::<Vec<_>>();

        layout.allocate(&children, width, height, baseline);
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
        let (mut min, mut nat, mut min_baseline, mut nat_baseline) = (0, 0, -1, -1);

        let mut curr = widget.first_child();

        while let Some(child) = curr {
            curr = child.next_sibling();

            if !child.should_layout() {
                continue;
            }

            let (c_min, c_nat, c_min_baseline, c_nat_baseline) =
                child.measure(orientation, for_size);

            min = c_min.max(min);
            nat = c_nat.max(nat);

            if c_min_baseline != -1 {
                min_baseline = min_baseline.max(c_min_baseline);
            }

            if c_nat_baseline != -1 {
                nat_baseline = nat_baseline.max(c_nat_baseline);
            }
        }

        let aspect = self.aspect_ratio.get();

        match (for_size, orientation) {
            (-1, gtk4::Orientation::Horizontal) => {
                let expected_width = widget.height() as f64 * aspect;
                nat = nat.min(expected_width.round() as i32).max(min);
            }
            (-1, gtk4::Orientation::Vertical) => {
                let expected_height = widget.width() as f64 / aspect;
                nat = nat.min(expected_height.round() as i32).max(min);
            }
            (s, gtk4::Orientation::Horizontal) => {
                let expected_width = s as f64 * aspect;
                nat = nat.min(expected_width.round() as i32).max(min);
            }
            (s, gtk4::Orientation::Vertical) => {
                let expected_height = s as f64 / aspect;
                nat = nat.min(expected_height.round() as i32).max(min);
            }
            _ => {}
        }

        (min, nat, min_baseline, nat_baseline)
    }
}

impl LayoutWleaveMenu {
    pub fn new(
        ratio: Option<impl Into<f64>>,
        column_spacing: impl Into<f64>,
        row_spacing: impl Into<f64>,
    ) -> Self {
        glib::Object::builder()
            .property("aspect-ratio-set", ratio.is_some())
            .property("aspect-ratio", ratio.map(Into::into).unwrap_or(1.0))
            .property("column-spacing", column_spacing.into())
            .property("row-spacing", row_spacing.into())
            .build()
    }
}

#[derive(Default)]
struct MenuLayout {
    strategy: MenuLayoutStrategy,
    column_spacing: f64,
    row_spacing: f64,
    aspect_ratio: Option<f64>,
}

#[derive(Default)]
enum MenuLayoutStrategy {
    #[default]
    Grid,
}

impl MenuLayout {
    fn allocate(&self, children: &[gtk4::Widget], width: i32, height: i32, baseline: i32) {
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

                // Axis-aligned rectangle packing
                // We brute-force the best layout, optimizing for max button area
                for i_rows in 1..=n {
                    for j_cols in 1..=n {
                        if i_rows * j_cols > n + i_rows
                            || i_rows * j_cols > n + j_cols
                            || i_rows * j_cols < n
                        {
                            continue;
                        }

                        let col_gaps = j_cols - 1;
                        let row_gaps = i_rows - 1;

                        let (w, h) = match self.aspect_ratio {
                            Some(aspect @ 1.0..) => {
                                let mut w = (width as usize - (col_gaps * col_spacing)) as f64
                                    / j_cols as f64
                                    * aspect;
                                let h = ((height as usize - (row_gaps * row_spacing)) as f64
                                    / i_rows as f64)
                                    .min(w / aspect);

                                w = h * aspect;

                                (w, h)
                            }
                            Some(aspect @ ..1.0) => {
                                let mut h = (height as usize - (row_gaps * row_spacing)) as f64
                                    / i_rows as f64;
                                let w = ((width as usize - (col_gaps * col_spacing)) as f64
                                    / j_cols as f64
                                    * aspect)
                                    .min(h * aspect);

                                h = w / aspect;

                                (w, h)
                            }
                            //
                            Some(..) | None => {
                                let w = (width as usize - (col_gaps * col_spacing)) as f64
                                    / j_cols as f64;
                                let h = (height as usize - (row_gaps * row_spacing)) as f64
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
                    if child.should_layout() {
                        let x_grid = (i % cols) as f64;
                        let y_grid = (i / cols) as f64;

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
}
