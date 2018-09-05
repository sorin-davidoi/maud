#![feature(specialization)]

//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: https://maud.lambda.xyz/

#![doc(html_root_url = "https://docs.rs/maud/0.18.1")]

#[cfg(feature = "streaming")] extern crate futures;
#[cfg(feature = "actix-web")] extern crate actix_web;
#[cfg(feature = "iron")] extern crate iron;
#[cfg(feature = "rocket")] extern crate rocket;

extern crate maud_htmlescape;
extern crate maud_macros;

use std::fmt::{self, Write};

pub use maud_macros::{html, html_debug};

#[cfg(feature = "streaming")]
pub use maud_macros::{html_stream, html_stream_debug};

/// Represents a type that can be rendered as HTML.
///
/// If your type implements [`Display`][1], then it will implement this
/// trait automatically through a blanket impl.
///
/// [1]: https://doc.rust-lang.org/std/fmt/trait.Display.html
///
/// On the other hand, if your type has a custom HTML representation,
/// then you can implement `Render` by hand. To do this, override
/// either the `.render()` or `.render_to()` methods; since each is
/// defined in terms of the other, you only need to implement one of
/// them. See the example below.
///
/// # Minimal implementation
///
/// An implementation of this trait must override at least one of
/// `.render()` or `.render_to()`. Since the default definitions of
/// these methods call each other, not doing this will result in
/// infinite recursion.
///
/// # Example
///
/// ```rust
/// # #![feature(proc_macro_non_items)]
/// use maud::{html, Markup, Render};
///
/// /// Provides a shorthand for linking to a CSS stylesheet.
/// pub struct Stylesheet(&'static str);
///
/// impl Render for Stylesheet {
///     fn render(&self) -> Markup {
///         html! {
///             link rel="stylesheet" type="text/css" href=(self.0);
///         }
///     }
/// }
/// ```
pub trait Render {
    /// Renders `self` as a block of `Markup`.
    fn render(&self) -> Markup {
        let mut buffer = String::new();
        self.render_to(&mut buffer);
        PreEscaped(buffer)
    }

    /// Appends a representation of `self` to the given buffer.
    ///
    /// Its default implementation just calls `.render()`, but you may
    /// override it with something more efficient.
    ///
    /// Note that no further escaping is performed on data written to
    /// the buffer. If you override this method, you must make sure that
    /// any data written is properly escaped, whether by hand or using
    /// the [`Escaper`](struct.Escaper.html) wrapper struct.
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.render().into_string());
    }
}

impl<T: fmt::Display + ?Sized> Render for T {
    default fn render_to(&self, w: &mut String) {
        let _ = write!(Escaper::new(w), "{}", self);
    }
}

impl Render for String {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_str(self);
    }
}

impl Render for str {
    fn render_to(&self, w: &mut String) {
        let _ = Escaper::new(w).write_str(self);
    }
}

#[cfg(feature = "streaming")]
pub trait FutureRender {
    fn render_to(&self, stream: &mut futures::stream::FuturesOrdered<
        Box<futures::Future<Item = String, Error = String> + Send>
    >);
}

#[cfg(feature = "streaming")]
impl FutureRender for String {
    fn render_to(&self, stream: &mut futures::stream::FuturesOrdered<
        Box<futures::Future<Item = String, Error = String> + Send>
    >) {
        let mut escaped = String::with_capacity(self.len());
        let _ = Escaper::new(&mut escaped).write_str(self);
        stream.push(Box::new(futures::future::ok(escaped)));
    }
}

#[cfg(feature = "streaming")]
impl FutureRender for str {
    fn render_to(&self, stream: &mut futures::stream::FuturesOrdered<
        Box<futures::Future<Item = String, Error = String> + Send>
    >) {
        let mut escaped = String::with_capacity(self.len());
        let _ = Escaper::new(&mut escaped).write_str(self);
        stream.push(Box::new(futures::future::ok(escaped)));
    }
}

/// A wrapper that renders the inner value without escaping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreEscaped<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for PreEscaped<T> {
    fn render_to(&self, w: &mut String) {
        w.push_str(self.0.as_ref());
    }
}

impl<T: AsRef<str>> FutureRender for PreEscaped<T> {
    fn render_to(&self, stream: &mut futures::stream::FuturesOrdered<
        Box<futures::Future<Item = String, Error = String> + Send>
    >) {
        let mut escaped = String::with_capacity(self.0.as_ref().len());
        let _ = Escaper::new(&mut escaped).write_str(self.0.as_ref());
        stream.push(Box::new(futures::future::ok(escaped)));
    }
}

/// A block of markup is a string that does not need to be escaped.
///
/// The `html!` macro expands to an expression of this type.
pub type Markup = PreEscaped<String>;

impl<T: AsRef<str> + Into<String>> PreEscaped<T> {
    /// Converts the inner value to a string.
    pub fn into_string(self) -> String {
        self.0.into()
    }
}

impl<T: AsRef<str> + Into<String>> Into<String> for PreEscaped<T> {
    fn into(self) -> String {
        self.into_string()
    }
}

pub use maud_htmlescape::Escaper;

/// The literal string `<!DOCTYPE html>`.
///
/// # Example
///
/// A minimal web page:
///
/// ```rust
/// # #![feature(proc_macro_non_items)]
/// use maud::{DOCTYPE, html};
///
/// let markup = html! {
///     (DOCTYPE)
///     html {
///         head {
///             meta charset="utf-8";
///             title { "Test page" }
///         }
///         body {
///             p { "Hello, world!" }
///         }
///     }
/// };
/// ```
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");

#[cfg(feature = "iron")]
mod iron_support {
    use std::io;
    use iron::headers::ContentType;
    use iron::modifier::{Modifier, Set};
    use iron::modifiers::Header;
    use iron::response::{Response, WriteBody};
    use PreEscaped;

    impl Modifier<Response> for PreEscaped<String> {
        fn modify(self, response: &mut Response) {
            response
                .set_mut(Header(ContentType::html()))
                .set_mut(Box::new(self) as Box<WriteBody>);
        }
    }

    impl WriteBody for PreEscaped<String> {
        fn write_body(&mut self, body: &mut io::Write) -> io::Result<()> {
            self.0.write_body(body)
        }
    }
}

#[cfg(feature = "rocket")]
mod rocket_support {
    use rocket::http::{ContentType, Status};
    use rocket::request::Request;
    use rocket::response::{Responder, Response};
    use std::io::Cursor;
    use PreEscaped;

    impl Responder<'static> for PreEscaped<String> {
        fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
            Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(self.0))
                .ok()
        }
    }
}

#[cfg(feature = "actix-web")]
mod actix_support {
    use PreEscaped;
    use actix_web::{Responder, HttpResponse, HttpRequest, Error};

    impl Responder for PreEscaped<String> {
        type Item = HttpResponse;
        type Error = Error;
        fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<Self::Item, Self::Error> {
            Ok(HttpResponse::Ok().body(self.0))
        }
    }
}
