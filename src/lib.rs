//! # Quill
//!
//! Quill provides an extension to ``TOML`` that allows for [Scope]
//! these are chunks of files which can be extracted, essentially permitting
//! multiple TOML files with shared components inside of one.
//!
//! No ``TOML`` parsing is done by Quill, it simply extracts [Scope] from the
//! file as requested by the user of the library and returns the TOML content
//! with the scope extracted.
//!
//! [Scope]'s are defined in the file by prepending a non-whitespaced string that
//! may only contain ASCII letters, ASCII digits, underscores, and dashes with a ``@``
//!
//! Any other content on the line of defined [Scope]'s will be ignored.
//!
//! For example,
//!
//! ```toml
//! @my_scope
//! ```
//!
//! By default all of the elements defined prior to a [Scope] being declared, fall into
//! the ``global`` scope, All ``global`` scope elements will be included on lookup of any
//! other [Scope].
//!
//! [Scope]'s can be used multiple times in one file, each definition refers to the same
//! [Scope] and will be returned in the output targeting that one [Scope].
//!
//! Multiple [Scope]'s can be declared on the same line to mark the proceeding content as
//! under both scopes equivalently, thus for retrieving any of the [Scope]'s on the same line,
//! the proceeding content will be included.
//!
//! # Example
//!
//! ```
//! use quill::{extract_scope, Scope};
//!
//! let toml = r#"
//! title = "App"
//!
//! @dev
//! debug = true
//!
//! @prod
//! optimized = true
//!
//! @dev @test
//! extra_checks = true
//!
//! @global
//! do_tests = true"#;
//!
//! let dev_config = extract_scope(toml, Scope::DefinedScope("dev")).unwrap();
//! assert_eq!(dev_config, r#"
//! title = "App"
//!
//!
//! debug = true
//!
//!
//!
//!
//!
//! extra_checks = true
//!
//!
//! do_tests = true"#);
//! ```

use std::fmt;

/// Types of scopes within Quill accessible
/// in a file
pub enum Scope<'a> {
    /// Global scope, non-scoped elements in file fall
    /// automatically into this, ALL global scope elements
    /// will automatically be included in every [Scope::DefinedScope]
    Global,
    /// A user-defined scope, must not contain spaces
    DefinedScope(&'a str),
}

impl<'a> From<&'a str> for Scope<'a> {
    fn from(value: &'a str) -> Self {
        Self::DefinedScope(value)
    }
}

impl<'a> Into<&'a str> for Scope<'a> {
    fn into(self) -> &'a str {
        // Into the name of the scope itself.
        match self {
            Scope::Global => "global",
            Scope::DefinedScope(name) => name,
        }
    }
}

/// Error type for Quill operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuillError {
    /// Invalid [Scope] name in the source file
    InvalidScopeName {
        scope: String,
        line: usize,
        column: usize,
    },
    /// Invalid [Scope] name provided as argument
    InvalidScopeArgument { scope: String },
}

impl fmt::Display for QuillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuillError::InvalidScopeName {
                scope,
                line,
                column,
            } => {
                write!(
                    f,
                    "Invalid scope name '{}' at line {}, column {}. Scope names may only contain ASCII letters, ASCII digits, underscores, and dashes.",
                    scope, line, column
                )
            }
            QuillError::InvalidScopeArgument { scope } => {
                write!(
                    f,
                    "Invalid scope name '{}'. Scope names may only contain ASCII letters, ASCII digits, underscores, and dashes.",
                    scope
                )
            }
        }
    }
}

impl std::error::Error for QuillError {}

/// Validates that a name matches the required [Scope]
/// name as per spec, which is that it only contains
/// ASCII letters, ASCII digits, underscores, and dashes
fn is_valid_scope_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Extracts the provided [Scope] from the provided
/// toml str
///
/// A extracted scope will have all of the [Scope] identifiers
/// removed from the file, and every other scope will have it's contents
/// emptied, thus extracting the correct [Scope] while preserving file
/// position for providing debugging information with location to the user
/// if needed.
///
/// Read more about [Scope]'s in the crate level documentation.
///
/// the returned content will be an allocated [String] from this [Scope]'s content
///
/// # Example
///
/// ```
/// use quill::{extract_scope, Scope};
///
/// let toml = r#"
/// title = "App"
///
/// @dev
/// debug = true
///
/// @prod
/// optimized = true
///
/// @dev @test
/// extra_checks = true
///
/// @global
/// do_tests = true"#;
///
/// let dev_config = extract_scope(toml, Scope::DefinedScope("dev")).unwrap();
/// assert_eq!(dev_config, r#"
/// title = "App"
///
///
/// debug = true
///
///
///
///
///
/// extra_checks = true
///
///
/// do_tests = true"#);
/// ```
pub fn extract_scope<'a, 'b, T: Into<&'a str>>(
    toml_str: T,
    scope: Scope<'b>,
) -> Result<String, QuillError> {
    // Extract args into string
    let toml_str = toml_str.into();
    let target_scope: &str = scope.into();

    // Validate the target scope name (unless it's "global")
    if target_scope.ne(Scope::Global.into()) && !is_valid_scope_name(target_scope) {
        return Err(QuillError::InvalidScopeArgument {
            scope: target_scope.to_string(),
        });
    }

    // String that will be returned at end on success.
    let mut result = String::with_capacity(toml_str.len());
    let mut lines = toml_str.lines();
    let mut current_scopes: Vec<&str> = vec![Scope::Global.into()];
    let mut include_content = true;
    let mut line_number = 0;

    while let Some(line) = lines.next() {
        line_number += 1;
        let trimmed = line.trim_start();

        // Check if this line contains scope declarations
        if trimmed.starts_with('@') {
            // Extract all scopes from this line
            let mut scopes: Vec<&str> = Vec::new();

            for token in trimmed.split_whitespace() {
                if token.starts_with('@') {
                    let scope_name = &token[1..];

                    // Validate scope name
                    if !is_valid_scope_name(scope_name) {
                        // Calculate column number (position of @ symbol)
                        let column = line.find('@').unwrap_or(0) + 1;
                        return Err(QuillError::InvalidScopeName {
                            scope: scope_name.to_string(),
                            line: line_number,
                            column,
                        });
                    }

                    scopes.push(scope_name);
                }
            }

            if !scopes.is_empty() {
                current_scopes = scopes;
                // Check if any of the declared scopes match our target
                include_content = current_scopes.contains(&target_scope)
                    || current_scopes.contains(&Scope::Global.into())
                    || target_scope.eq(Scope::Global.into());

                // Add empty line to preserve line numbers
                result.push('\n');
                continue;
            }
        }

        // Include or exclude content based on current scope
        if include_content {
            result.push_str(line);
        } else {
            // Replace content with empty space to preserve line numbers
            // Keep the newline structure
        }
        result.push('\n');
    }

    // Remove the trailing newline if the original didn't have one
    if !toml_str.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    Ok(result)
}
