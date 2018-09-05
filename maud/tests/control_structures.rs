#![feature(proc_macro_non_items)]

#[cfg(feature = "streaming")]
extern crate futures;
extern crate maud;

#[cfg(feature = "streaming")]
use futures::Stream;
use maud::html;
#[cfg(feature = "streaming")]
use maud::html_stream_debug;

#[test]
fn if_expr() {
    for (number, &name) in (1..4).zip(["one", "two", "three"].iter()) {
        let s = html! {
            @if number == 1 {
                "one"
            } @else if number == 2 {
                "two"
            } @else if number == 3 {
                "three"
            } @else {
                "oh noes"
            }
        }.into_string();
        assert_eq!(s, name);
    }
}

#[cfg(feature = "streaming")]
#[test]
fn if_expr_stream() {
    for (number, &name) in (1..4).zip(["one", "two", "three"].iter()) {
        let mut s = html_stream_debug! {
            @if number == 1 {
                "one"
            } @else if number == 2 {
                "two"
            } @else if number == 3 {
                "three"
            } @else {
                "oh noes"
            }
        }.wait();
        assert_eq!(s.next(), Some(Ok(name)));
        assert_eq!(s.next(), None);
    }
}

#[test]
fn if_expr_in_class() {
    for &(chocolate_milk, expected) in &[
        (0, r#"<p class="empty">Chocolate milk</p>"#),
        (1, r#"<p class="full">Chocolate milk</p>"#),
    ]
    {
        let s = html! {
            p.@if chocolate_milk == 0 { "empty" } @else { "full" } {
                "Chocolate milk"
            }
        }.into_string();
        assert_eq!(s, expected);
    }
}

#[test]
fn if_let() {
    for &(input, output) in &[(Some("yay"), "yay"), (None, "oh noes")] {
        let s = html! {
            @if let Some(value) = input {
                (value)
            } @else {
                "oh noes"
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn while_expr() {
    let mut numbers = (0..3).into_iter().peekable();
    let s = html! {
        ul {
            @while numbers.peek().is_some() {
                li { (numbers.next().unwrap()) }
            }
        }
    }.into_string();
    assert_eq!(s, "<ul><li>0</li><li>1</li><li>2</li></ul>");
}

#[test]
fn while_let_expr() {
    let mut numbers = (0..3).into_iter();
    let s = html! {
        ul {
            @while let Some(n) = numbers.next() {
                li { (n) }
            }
        }
    }.into_string();
    assert_eq!(s, "<ul><li>0</li><li>1</li><li>2</li></ul>");
}

#[test]
fn for_expr() {
    let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"];
    let s = html! {
        ul {
            @for pony in &ponies {
                li { (pony) }
            }
        }
    }.into_string();
    assert_eq!(s, concat!(
            "<ul>",
            "<li>Apple Bloom</li>",
            "<li>Scootaloo</li>",
            "<li>Sweetie Belle</li>",
            "</ul>"));
}

// #[cfg(feature = "streaming")]
// #[test]
// fn for_expr_stream() {
//     let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"];
//     let mut s = html_stream_debug! {
//         ul {
//             @for pony in &ponies {
//                 li { (pony) }
//             }
//         }
//     }.wait();
    
//     assert_eq!(s.next(), Some(Ok("<ul>")));
//     assert_eq!(s.next(), Some(Ok("<li>Apple Bloom</li>")));
//     assert_eq!(s.next(), Some(Ok("<li>Scootaloo</li>")));
//     assert_eq!(s.next(), Some(Ok("<li>Sweetie Belle</li>")));
//     assert_eq!(s.next(), Some(Ok("</ul>")));
//     assert_eq!(s.next(), None);
// }

#[test]
fn match_expr() {
    for &(input, output) in &[(Some("yay"), "<div>yay</div>"), (None, "oh noes")] {
        let s = html! {
            @match input {
                Some(value) => {
                    div { (value) }
                },
                None => {
                    "oh noes"
                },
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn match_expr_without_delims() {
    for &(input, output) in &[(Some("yay"), "yay"), (None, "<span>oh noes</span>")] {
        let s = html! {
            @match input {
                Some(value) => (value),
                None => span { "oh noes" },
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn match_no_trailing_comma() {
    for &(input, output) in &[(Some("yay"), "yay"), (None, "<span>oh noes</span>")] {
        let s = html! {
            @match input {
                Some(value) => { (value) }
                None => span { "oh noes" }
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn match_expr_with_guards() {
    for &(input, output) in &[(Some(1), "one"), (None, "none"), (Some(2), "2")] {
        let s = html! {
            @match input {
                Some(value) if value == 1 => "one",
                Some(value) => (value),
                None => "none",
            }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn match_in_attribute() {
    for &(input, output) in &[(1, "<span class=\"one\">1</span>"), (2, "<span class=\"two\">2</span>"), (3, "<span class=\"many\">3</span>")] {
        let s = html! {
            span class=@match input {
                1 => "one",
                2 => "two",
                _ => "many",
            } { (input) }
        }.into_string();
        assert_eq!(s, output);
    }
}

#[test]
fn let_expr() {
    let s = html! {
        @let x = 42;
        "I have " (x) " cupcakes!"
    }.into_string();
    assert_eq!(s, "I have 42 cupcakes!");
}

#[test]
fn let_lexical_scope() {
    let x = 42;
    let s = html! {
        {
            @let x = 99;
            "Twilight thought I had " (x) " cupcakes, "
        }
        "but I only had " (x) "."
    }.into_string();
    assert_eq!(s, concat!(
            "Twilight thought I had 99 cupcakes, ",
            "but I only had 42."));
}

#[test]
fn let_type_ascription() {
    let s = html! {
        @let mut x: Box<Iterator<Item=u32>> = Box::new(vec![42].into_iter());
        "I have " (x.next().unwrap()) " cupcakes!"
    }.into_string();
    assert_eq!(s, "I have 42 cupcakes!");
}
