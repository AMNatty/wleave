use gdk4::prelude::*;
use gdk4::subclass::paintable::PaintableImpl;
use glib::Object;
use glib::subclass::ObjectImplRef;
use glib::subclass::prelude::*;
use glib::translate::IntoGlib;
use glib_macros::Properties;
use gtk4::prelude::*;
use gtk4::subclass::prelude::SymbolicPaintableImpl;
use miette::miette;
use std::cell::{Cell, RefCell};
use tracing::error;

#[derive(Properties, Default)]
#[properties(wrapper_type = PicturePaintable)]
pub struct PicturePaintableImpl {
    #[property(name = "image-path", get, set)]
    image_path: RefCell<String>,
    #[property(name = "widget", get, set)]
    widget: RefCell<Option<gtk4::Widget>>,
    handle: RefCell<Option<rsvg::SvgHandle>>,
    texture: RefCell<Option<gdk4::MemoryTexture>>,
    _symbolic_updated: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for PicturePaintableImpl {
    const NAME: &'static str = "WleavePicturePaintable";

    type Type = PicturePaintable;

    type ParentType = glib::Object;

    type Interfaces = (gdk4::Paintable, gtk4::SymbolicPaintable);
}

glib::wrapper! {
    pub struct PicturePaintable(ObjectSubclass<PicturePaintableImpl>)
        @implements gdk4::Paintable, gtk4::SymbolicPaintable;
}

#[glib::derived_properties]
impl ObjectImpl for PicturePaintableImpl {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        let impl_ref = ObjectImplRef::new(self);
        obj.connect_notify_local(Some("image-path"), move |pict, _| {
            impl_ref.texture.take();

            *impl_ref.handle.borrow_mut() = match rsvg::Loader::new()
                .read_path(pict.image_path())
                .map_err(|e| miette!("Failed to read SVG: {}", e))
            {
                Ok(handle) => Some(handle),
                Err(e) => {
                    error!("{}", e);
                    None
                }
            };
        });
    }
}

impl PicturePaintableImpl {
    fn draw(&self, width: f64, height: f64) {
        let scale = self
            .widget
            .borrow()
            .as_ref()
            .map(|w| w.scale_factor() as f64)
            .unwrap_or(1.0);
        let height = height * scale;
        let width = width * scale;
        let mut tex_borrow = self.texture.borrow_mut();
        if tex_borrow.is_some() {
            return;
        };

        let Some(handle_ref) = &*self.handle.borrow() else {
            return;
        };

        let renderer = rsvg::CairoRenderer::new(handle_ref);

        let mut surface =
            match cairo::ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32)
                .map_err(|e| miette!("Failed to create a Cairo surface: {}", e))
            {
                Ok(surf) => surf,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };

        let ctx = match cairo::Context::new(&surface)
            .map_err(|e| miette!("Failed to create a Cairo context: {}", e))
        {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        if let Err(e) = renderer
            .render_document(&ctx, &cairo::Rectangle::new(0.0, 0.0, width, height))
            .map_err(|e| miette!("Failed to render SVG: {}", e))
        {
            error!("{}", e);
            return;
        }

        drop(ctx);

        let data = match surface
            .data()
            .map_err(|e| miette!("Failed to take Cairo image data: {}", e))
        {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        let bytes = glib::Bytes::from(data.as_ref());

        cfg_if::cfg_if! {
            if #[cfg(target_endian = "little")] {
                let format = gdk4::MemoryFormat::B8g8r8a8;
            } else {
                let format = gdk4::MemoryFormat::A8r8g8b8;
            }
        };

        *tex_borrow = Some(gdk4::MemoryTexture::new(
            width as i32,
            height as i32,
            format,
            &bytes,
            size_of::<i32>() * width as usize,
        ));
    }
}

impl PaintableImpl for PicturePaintableImpl {
    fn current_image(&self) -> gdk4::Paintable {
        self.draw(
            self.intrinsic_width() as f64,
            self.intrinsic_height() as f64,
        );

        let Some(tex_ref) = &*self.texture.borrow() else {
            return gdk4::Paintable::new_empty(self.intrinsic_width(), self.intrinsic_height());
        };

        gdk4::Paintable::from(tex_ref.clone())
    }

    fn flags(&self) -> gdk4::PaintableFlags {
        gdk4::PaintableFlags::empty()
    }

    fn intrinsic_width(&self) -> i32 {
        let Some(handle_ref) = &*self.handle.borrow() else {
            return 256;
        };

        let renderer = rsvg::CairoRenderer::new(handle_ref);
        let size = renderer
            .intrinsic_size_in_pixels()
            .map(|(w, _)| w.ceil() as i32);

        size.unwrap_or(256)
    }

    fn intrinsic_height(&self) -> i32 {
        let Some(handle_ref) = &*self.handle.borrow() else {
            return 256;
        };

        let renderer = rsvg::CairoRenderer::new(handle_ref);
        let size = renderer
            .intrinsic_size_in_pixels()
            .map(|(_, h)| h.ceil() as i32);

        size.unwrap_or(256)
    }

    fn intrinsic_aspect_ratio(&self) -> f64 {
        let Some(handle_ref) = &*self.handle.borrow() else {
            return 1.0;
        };

        let renderer = rsvg::CairoRenderer::new(handle_ref);
        let size = renderer.intrinsic_size_in_pixels().map(|(w, h)| w / h);

        size.unwrap_or(1.0)
    }

    fn snapshot(&self, snapshot: &gdk4::Snapshot, width: f64, height: f64) {
        if !self._symbolic_updated.get() {
            if let Some(widget) = &*self.widget.borrow() {
                self.snapshot_symbolic(snapshot, width, height, &[widget.color()]);
            } else {
                self.snapshot_symbolic(snapshot, width, height, &[]);
            }
        }

        self.draw(width, height);

        let Some(tex_ref) = &*self.texture.borrow() else {
            return;
        };

        SnapshotExt::append_texture(
            snapshot,
            tex_ref,
            &gtk4::graphene::Rect::new(0.0, 0.0, width as f32, height as f32),
        );

        self._symbolic_updated.set(false);
    }
}

impl SymbolicPaintableImpl for PicturePaintableImpl {
    fn snapshot_symbolic(
        &self,
        snapshot: &gdk4::Snapshot,
        width: f64,
        height: f64,
        colors: &[gdk4::RGBA],
    ) {
        let mut handle_borrow = self.handle.borrow_mut();
        let Some(handle_ref) = &mut *handle_borrow else {
            return;
        };

        let col_idx = gtk4::SymbolicColor::Foreground.into_glib();

        let col = colors[col_idx as usize];

        if let Err(e) = handle_ref
            .set_stylesheet(&format!(
                r#"
                    svg {{
                        color: {col} !important;
                    }}
                "#
            ))
            .map_err(|e| miette!("Failed to set stylesheet for SVG while loading: {}", e))
        {
            error!("{}", e);
            return;
        }

        drop(handle_borrow);

        self.texture.take();
        self._symbolic_updated.set(true);

        self.snapshot(snapshot, width, height);
    }
}

impl PicturePaintable {
    fn for_path(icon_path: impl Into<String>) -> Self {
        Object::builder()
            .property("image-path", icon_path.into())
            .build()
    }
}

pub fn svg_picture_colorized(icon: &str) -> gtk4::Picture {
    let paintable = PicturePaintable::for_path(icon);
    let picture = gtk4::Picture::for_paintable(&paintable);
    paintable.set_widget(picture.clone());
    picture
}
