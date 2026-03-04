use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_ast::{self as ast, Expr, Stmt};
use ruff_text_size::Ranged;

use crate::Violation;
use crate::checkers::ast::Checker;
use crate::rules::flake8_django::helpers::is_concrete_model_serializer;

/// ## What it does
/// Checks for the use of `exclude` in Django REST Framework ModelSerializers.
///
/// ## Why is this bad?
/// Using `exclude` can lead to implicit behavior and potential security issues
/// if new fields are added to the model and inadvertently exposed via the serializer.
/// It's safer to explicitly list fields using `fields`.
///
/// ## Example
/// ```python
/// from rest_framework import serializers
/// from myapp.models import MyModel
///
/// class MyModelSerializer(serializers.ModelSerializer):
///     class Meta:
///         model = MyModel
///         exclude = ["id"]  # Bad: implicit field listing
/// ```
///
/// Use instead:
/// ```python
/// class MyModelSerializer(serializers.ModelSerializer):
///     class Meta:
///         model = MyModel
///         fields = ["name", "description"]  # Good: explicit
/// ```
#[derive(ViolationMetadata)]
#[violation_metadata(stable_since = "v0.0.253")]  // or whatever version you're targeting
pub(crate) struct ExcludeInModelSerializer;

impl Violation for ExcludeInModelSerializer {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Avoid using `exclude` in a `ModelSerializer`, prefer explicit `fields`".to_string()
    }
}

/// DJ017
pub(crate) fn exclude_with_serializer(checker: &Checker, class_def: &ast::StmtClassDef) {
    // Optional: check if Django was imported at all (like in exclude_with_model_form)
    // if !checker.semantic().seen_module(Modules::DJANGO) {
    //     return;
    // }

    if !is_concrete_model_serializer(class_def, checker.semantic()) {
        return;
    }

    for element in &class_def.body {
        let Stmt::ClassDef(ast::StmtClassDef { name, body, .. }) = element else {
            continue;
        };
        if name != "Meta" {
            continue;
        }
        for element in body {
            let Stmt::Assign(ast::StmtAssign { targets, .. }) = element else {
                continue;
            };
            for target in targets {
                let Expr::Name(ast::ExprName { id, .. }) = target else {
                    continue;
                };
                if id == "exclude" {
                    checker.report_diagnostic(ExcludeInModelSerializer, target.range());
                    return;
                }
            }
        }
    }
}