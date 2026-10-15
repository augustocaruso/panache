use panache_formatter::config::{Config, Flavor, FormatterExtensions, ParserExtensions};
use panache_formatter::format;

fn config_for_flavor(flavor: Flavor) -> Config {
    Config {
        flavor,
        parser_extensions: ParserExtensions::for_flavor(flavor),
        formatter_extensions: FormatterExtensions::for_flavor(flavor),
        ..Config::default()
    }
}

#[test]
fn heading_expands_unnumbered_shorthand_in_pandoc_flavors() {
    let input = "# Heading {-}\n\n## Heading {.unlisted - #section}\n\nSetext {-}\n----------\n\n- # Nested {-}\n";
    let expected = "# Heading {.unnumbered}\n\n## Heading {#section .unlisted .unnumbered}\n\n## Setext {.unnumbered}\n\n- # Nested {.unnumbered}\n";
    for flavor in [Flavor::Pandoc, Flavor::Quarto, Flavor::RMarkdown] {
        let config = config_for_flavor(flavor);
        let output = format(input, Some(config.clone()), None);
        assert_eq!(output, expected, "{flavor:?}");
        assert_eq!(format(&output, Some(config), None), output, "{flavor:?}");
    }
}

#[test]
fn heading_preserves_unnumbered_spelling_in_other_flavors() {
    let input = "# Heading {-}\n\n# Heading {.unnumbered}\n";
    for flavor in [
        Flavor::CommonMark,
        Flavor::Gfm,
        Flavor::Hugo,
        Flavor::MultiMarkdown,
        Flavor::Mdsvex,
        Flavor::Myst,
    ] {
        let mut config = config_for_flavor(flavor);
        assert_eq!(
            format(input, Some(config.clone()), None),
            input,
            "{flavor:?}"
        );
        config.parser_extensions.header_attributes = true;
        assert_eq!(format(input, Some(config), None), input, "{flavor:?}");
    }
}

#[test]
fn heading_shorthand_is_literal_when_attributes_are_disabled() {
    let input = "# Heading {-}\n";
    let mut config = config_for_flavor(Flavor::Pandoc);
    config.parser_extensions.header_attributes = false;
    assert_eq!(format(input, Some(config), None), input);
}

#[test]
fn heading_with_simple_id() {
    let input = "# Heading {#my-id}\n";
    let output = format(input, None, None);
    assert!(output.contains("# Heading {#my-id}"));
}

#[test]
fn heading_with_single_class() {
    let input = "# Heading {.myclass}\n";
    let output = format(input, None, None);
    assert!(output.contains("# Heading {.myclass}"));
}

#[test]
fn heading_with_multiple_classes() {
    let input = "## Section {.class1 .class2 .class3}\n";
    let output = format(input, None, None);
    assert!(output.contains("## Section {.class1 .class2 .class3}"));
}

#[test]
fn heading_with_key_value() {
    let input = "# Title {key=value}\n";
    let output = format(input, None, None);
    assert!(output.contains("# Title {key=\"value\"}"));
}

#[test]
fn heading_with_quoted_value() {
    let input = "# Title {key=\"value with spaces\"}\n";
    let output = format(input, None, None);
    assert!(output.contains("# Title {key=\"value with spaces\"}"));
}

#[test]
fn heading_with_full_attributes() {
    let input = "# Heading {#id .class1 .class2 key1=val1 key2=\"val 2\"}\n";
    let output = format(input, None, None);
    assert!(output.contains("# Heading {#id .class1 .class2 key1=\"val1\" key2=\"val 2\"}"));
}

#[test]
fn multiple_headings_with_attributes() {
    let input = "# First {#first}\n\n## Second {#second .important}\n\n### Third {.section}\n";
    let output = format(input, None, None);
    assert!(output.contains("# First {#first}"));
    assert!(output.contains("## Second {#second .important}"));
    assert!(output.contains("### Third {.section}"));
}

#[test]
fn heading_without_attributes() {
    let input = "# Plain Heading\n";
    let output = format(input, None, None);
    assert!(output.contains("# Plain Heading"));
    assert!(!output.contains("{"));
}

#[test]
fn heading_with_trailing_hashes_and_attributes() {
    let input = "# Heading ### {#id}\n";
    let output = format(input, None, None);
    assert!(output.contains("# Heading {#id}"));
}

#[test]
fn level_2_heading_with_attributes() {
    let input = "## Section Title {#sec-intro .unnumbered}\n";
    let output = format(input, None, None);
    assert!(output.contains("## Section Title {#sec-intro .unnumbered}"));
}

#[test]
fn heading_preserves_whitespace_in_attributes() {
    let input = "# Title {  #id   .class   key=val  }\n";
    let output = format(input, None, None);
    assert!(output.contains("# Title {#id .class key=\"val\"}"));
}

#[test]
fn heading_with_mixed_content() {
    let input = "# Introduction to *Pandoc* {#intro}\n\nSome text.\n";
    let output = format(input, None, None);
    assert!(output.contains("# Introduction to *Pandoc* {#intro}"));
}

#[test]
fn idempotent_formatting() {
    let input = "# Heading {#id .class key=value}\n";
    let first = format(input, None, None);
    let second = format(&first, None, None);
    assert_eq!(first, second);
}
