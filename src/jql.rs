use std::fmt;
use std::fmt::Write;

/// JQL field names supported by the builder.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Field {
    Assignee,
    Reporter,
    Status,
    StatusCategory,
    Project,
    Priority,
    Type,
    Labels,
    Sprint,
    Resolution,
    Created,
    Updated,
    Due,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Field::Assignee => "assignee",
            Field::Reporter => "reporter",
            Field::Status => "status",
            Field::StatusCategory => "statusCategory",
            Field::Project => "project",
            Field::Priority => "priority",
            Field::Type => "type",
            Field::Labels => "labels",
            Field::Sprint => "sprint",
            Field::Resolution => "resolution",
            Field::Created => "created",
            Field::Updated => "updated",
            Field::Due => "due",
        };
        f.write_str(name)
    }
}

/// Comparison operators for JQL conditions.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Operator {
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    NotContains,
    Is,
    IsNot,
    In,
    NotIn,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            Operator::Eq => "=",
            Operator::NotEq => "!=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::Contains => "~",
            Operator::NotContains => "!~",
            Operator::Is => "IS",
            Operator::IsNot => "IS NOT",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
        };
        f.write_str(op)
    }
}

/// Right-hand side value in a JQL condition.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    /// A quoted string value, e.g. `"Done"`
    Str(String),
    /// A JQL function call, e.g. `currentUser()`
    Func(String),
    /// EMPTY / NULL keyword
    Empty,
    /// A list of values for IN / NOT IN, e.g. `("a", "b")`
    List(Vec<String>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::Func(func) => f.write_str(func),
            Value::Empty => f.write_str("EMPTY"),
            Value::List(items) => {
                f.write_char('(')?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "\"{}\"", item)?;
                }
                f.write_char(')')
            }
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => f.write_str("ASC"),
            SortOrder::Desc => f.write_str("DESC"),
        }
    }
}

/// Boolean conjunction between conditions.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum Conjunction {
    And,
    Or,
}

impl fmt::Display for Conjunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Conjunction::And => f.write_str(" AND "),
            Conjunction::Or => f.write_str(" OR "),
        }
    }
}

#[derive(Debug, Clone)]
struct Condition {
    conjunction: Option<Conjunction>,
    field: Field,
    operator: Operator,
    value: Value,
}

#[derive(Debug, Clone)]
struct OrderByClause {
    field: Field,
    order: SortOrder,
}

/// Builder for constructing JQL query strings.
///
/// Validates project keys to prevent JQL injection.
#[derive(Debug, Clone)]
#[must_use = "JqlBuilder does nothing until .build() is called"]
pub struct JqlBuilder {
    conditions: Vec<Condition>,
    order_by: Vec<OrderByClause>,
}

impl JqlBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            order_by: Vec::new(),
        }
    }

    /// Add a condition joined by AND (or as the first condition).
    pub fn and(mut self, field: Field, op: Operator, value: Value) -> Self {
        let conjunction = if self.conditions.is_empty() {
            None
        } else {
            Some(Conjunction::And)
        };
        self.conditions.push(Condition {
            conjunction,
            field,
            operator: op,
            value,
        });
        self
    }

    /// Add a condition joined by OR.
    #[allow(dead_code)]
    pub fn or(mut self, field: Field, op: Operator, value: Value) -> Self {
        let conjunction = if self.conditions.is_empty() {
            None
        } else {
            Some(Conjunction::Or)
        };
        self.conditions.push(Condition {
            conjunction,
            field,
            operator: op,
            value,
        });
        self
    }

    /// Shortcut: `assignee = currentUser()`
    pub fn assignee_is_current_user(self) -> Self {
        self.and(
            Field::Assignee,
            Operator::Eq,
            Value::Func("currentUser()".into()),
        )
    }

    /// Shortcut: `statusCategory != "Done"`
    pub fn exclude_done(self) -> Self {
        self.and(
            Field::StatusCategory,
            Operator::NotEq,
            Value::Str("Done".into()),
        )
    }

    /// Shortcut: `project = "KEY"` with validation.
    ///
    /// Returns `Err` if the project key contains invalid characters.
    pub fn project(self, key: &str) -> Result<Self, JqlBuildError> {
        validate_project_key(key)?;
        Ok(self.and(Field::Project, Operator::Eq, Value::Str(key.into())))
    }

    /// Add ORDER BY clause.
    pub fn order_by(mut self, field: Field, order: SortOrder) -> Self {
        self.order_by.push(OrderByClause { field, order });
        self
    }

    /// Build the final JQL string.
    pub fn build(self) -> String {
        let mut jql = String::new();

        for cond in &self.conditions {
            if let Some(conj) = &cond.conjunction {
                let _ = write!(jql, "{}", conj);
            }
            let _ = write!(jql, "{} {} {}", cond.field, cond.operator, cond.value);
        }

        if !self.order_by.is_empty() {
            jql.push_str(" ORDER BY ");
            for (i, clause) in self.order_by.iter().enumerate() {
                if i > 0 {
                    jql.push_str(", ");
                }
                let _ = write!(jql, "{} {}", clause.field, clause.order);
            }
        }

        jql
    }
}

impl Default for JqlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct JqlBuildError {
    pub message: String,
}

impl fmt::Display for JqlBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JQL build error: {}", self.message)
    }
}

impl std::error::Error for JqlBuildError {}

/// Project keys must be uppercase alphanumeric + underscore only.
fn validate_project_key(key: &str) -> Result<(), JqlBuildError> {
    if key.is_empty() {
        return Err(JqlBuildError {
            message: "project key cannot be empty".into(),
        });
    }
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(JqlBuildError {
            message: format!(
                "project key '{}' contains invalid characters (only A-Z, 0-9, _ allowed)",
                key
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_assignee_query() {
        let jql = JqlBuilder::new().assignee_is_current_user().build();
        assert_eq!(jql, "assignee = currentUser()");
    }

    #[test]
    fn test_assignee_exclude_done() {
        let jql = JqlBuilder::new()
            .assignee_is_current_user()
            .exclude_done()
            .build();
        assert_eq!(
            jql,
            "assignee = currentUser() AND statusCategory != \"Done\""
        );
    }

    #[test]
    fn test_with_project_and_order() {
        let jql = JqlBuilder::new()
            .assignee_is_current_user()
            .exclude_done()
            .project("WEB")
            .unwrap()
            .order_by(Field::Updated, SortOrder::Desc)
            .build();
        assert_eq!(
            jql,
            "assignee = currentUser() AND statusCategory != \"Done\" AND project = \"WEB\" ORDER BY updated DESC"
        );
    }

    #[test]
    fn test_project_key_validation_rejects_injection() {
        let result =
            JqlBuilder::new().project("WEB\" OR assignee != currentUser() OR project = \"X");
        assert!(result.is_err());
    }

    #[test]
    fn test_project_key_validation_rejects_empty() {
        let result = JqlBuilder::new().project("");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_order_by() {
        let jql = JqlBuilder::new()
            .assignee_is_current_user()
            .order_by(Field::Priority, SortOrder::Desc)
            .order_by(Field::Updated, SortOrder::Desc)
            .build();
        assert_eq!(
            jql,
            "assignee = currentUser() ORDER BY priority DESC, updated DESC"
        );
    }

    #[test]
    fn test_or_conjunction() {
        let jql = JqlBuilder::new()
            .and(Field::Status, Operator::Eq, Value::Str("Open".into()))
            .or(Field::Status, Operator::Eq, Value::Str("Reopened".into()))
            .build();
        assert_eq!(jql, "status = \"Open\" OR status = \"Reopened\"");
    }

    #[test]
    fn test_in_operator() {
        let jql = JqlBuilder::new()
            .and(
                Field::Project,
                Operator::In,
                Value::List(vec!["WEB".into(), "API".into()]),
            )
            .build();
        assert_eq!(jql, "project IN (\"WEB\", \"API\")");
    }

    #[test]
    fn test_is_empty() {
        let jql = JqlBuilder::new()
            .and(Field::Labels, Operator::Is, Value::Empty)
            .build();
        assert_eq!(jql, "labels IS EMPTY");
    }
}
