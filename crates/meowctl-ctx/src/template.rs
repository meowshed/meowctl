//! `{{name}}` substitution, as `renderTemplate` does it.
//!
//! Not a template language. Substitution is what components use, and a
//! language would be a dependency and a surface; see [R-CTX-027].

use std::collections::BTreeMap;

/// Substitutes `{{key}}` for each entry.
///
/// Keys are applied in sorted order, which `v0.1.0` leaves to Go's map
/// iteration. The result differs only when one variable's value contains
/// another's placeholder, and an order nobody can predict is worse than one
/// somebody can.
#[must_use]
pub fn render(template: &str, variables: &BTreeMap<String, String>) -> String {
    let mut out = template.to_owned();
    for (key, value) in variables {
        out = out.replace(&format!("{{{{{key}}}}}"), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    /// [R-CTX-027] the shape `renderTemplate` accepts, which every component
    /// that renders a config file depends on.
    #[test]
    fn a_placeholder_is_replaced_everywhere_it_appears() {
        assert_eq!(
            render("{{name}} is {{name}}", &vars(&[("name", "git")])),
            "git is git"
        );
    }

    #[test]
    fn a_placeholder_with_no_variable_is_left_alone() {
        assert_eq!(render("{{absent}}", &vars(&[])), "{{absent}}");
    }

    #[test]
    fn a_template_with_no_variables_is_returned_as_it_is() {
        assert_eq!(render("plain text", &vars(&[])), "plain text");
    }

    /// Braces that are not a placeholder are ordinary text, which matters for
    /// every shell and JSON template a component renders.
    #[test]
    fn single_braces_are_not_placeholders() {
        let rendered = render("{ \"a\": {{v}} }", &vars(&[("v", "1")]));
        assert_eq!(rendered, "{ \"a\": 1 }");
    }
}
