//! This crate provides a convenient way of handling multiple
//! errors.
//!
//! Instead of returning early with the first error in your app,
//! it helps you store the errors that occur in a tree structure.
//! It lets you label the errors, and flatten then into a list
//! to present to the user.
use itertools::Itertools;

/// The error Tree structure.
///
/// - `L` is the Label type.
/// - `E` is the inner Error type. It can be an error enum (from the thiserror package).
#[derive(Debug)]
pub enum ErrorTree<L, E> {
    /// Stores your single error type.
    Leaf(E),
    /// Adds a label to the subtree.
    Edge(L, Box<ErrorTree<L, E>>),
    /// Groups multiple subtrees at the same level.
    Vec(Vec<ErrorTree<L, E>>),
}

impl<L, E> ErrorTree<L, E> {
    /**
    Creates a `Leaf` tree from an `error`.

    ```rust
    # use itertools::*;
    # use error_trees::*;
    struct Error(String);
    let error_tree = ErrorTree::<&'static str, _>::leaf(Error("error".into()));
    ```
    */
    pub fn leaf(error: E) -> Self {
        Self::Leaf(error)
    }
}

/// The flattened error type
#[derive(Debug)]
pub struct FlatError<L, E> {
    /// The path from the leaf to the root of the tree.
    pub path: Vec<L>,
    /// The error
    pub error: E,
}

impl<L, E> ErrorTree<L, E>
where
    L: Clone,
{
    /**
    Flattens the error tree in a `Vec` of `FlatError`s.

    ```rust
    # use itertools::*;
    # use error_trees::*;
    #[derive(Debug)]
    struct Error(String);

    let error_1 = ErrorTree::leaf(Error("error1".into())).with_label("label1");
    let error_2 = ErrorTree::leaf(Error("error2".into())).with_label("label2");

    let errors = vec![error_1, error_2];

    let tree: ErrorTree<&'static str, Error> = errors.into();
    let tree = tree.with_label("parent_label");

    let flat_errors = tree.flatten_tree();

    assert!(
        matches!(
            &flat_errors[..],
            [
                FlatError {
                    path: path1,
                    error: Error(error1),
                },
                FlatError {
                    path: path2,
                    error: Error(error2),
                },
            ]
            if path1 == &vec!["label1", "parent_label"]
            && path2 == &vec!["label2", "parent_label"]
            && error1 == "error1"
            && error2 == "error2"
        ),
        "unexpected: {:#?}",
        flat_errors
    );
    ```
    */
    pub fn flatten_tree(self) -> Vec<FlatError<L, E>> {
        match self {
            ErrorTree::Leaf(error) => vec![FlatError {
                path: Vec::new(),
                error,
            }],
            ErrorTree::Edge(label, tree) => {
                let mut flat_errors = tree.flatten_tree();
                for flat in &mut flat_errors {
                    flat.path.push(label.clone());
                }
                flat_errors
            }
            ErrorTree::Vec(errors) => errors
                .into_iter()
                .flat_map(|tree| tree.flatten_tree())
                .collect_vec(),
        }
    }
}

/// Adds a label to the error tree.
pub trait IntoErrorTree<L, E> {
    /**
    Adds a `label` to an error tree.
    ```rust
    # use error_trees::*;
    struct Error(String);
    let leaf = ErrorTree::leaf(Error("a regular error".into()));
    let labeled_leaf = leaf.with_label("the label");
    ```
    */
    fn with_label(self, label: L) -> ErrorTree<L, E>;
}

impl<L, E> IntoErrorTree<L, E> for E
where
    E: Into<ErrorTree<L, E>>,
{
    fn with_label(self, label: L) -> ErrorTree<L, E> {
        ErrorTree::Edge(label, Box::new(self.into()))
    }
}

impl<L, E> IntoErrorTree<L, E> for ErrorTree<L, E> {
    fn with_label(self, label: L) -> ErrorTree<L, E> {
        ErrorTree::Edge(label, Box::new(self))
    }
}

impl<L, E> From<Vec<ErrorTree<L, E>>> for ErrorTree<L, E> {
    fn from(subtrees: Vec<ErrorTree<L, E>>) -> Self {
        ErrorTree::Vec(subtrees)
    }
}

impl<L, E> From<Vec<E>> for ErrorTree<L, E>
where
    E: IntoErrorTree<L, E>,
    ErrorTree<L, E>: From<E>,
{
    fn from(errors: Vec<E>) -> Self {
        ErrorTree::Vec(errors.into_iter().map(|x| x.into()).collect_vec())
    }
}

/// Convenience trait to convert tuple of `(success: T, errors: Vec<E>)` to a `result : Result<T, ErrorTree<L, E>>`
pub trait IntoResult<T, E1, E2> {
    /**
    Turns `self` into a `Result`.

    For tuples of `(success: T, errors: Vec<E>)`:
    - It checks if `errors` is empty.
        - If true, it will return `Ok(success)`.
        - Otherwise it will return `Err(errors)`.

    ```rust
    # use itertools::*;
    # use error_trees::*;
    struct Error(String);

    let result1: Result<(), Error> = Err(Error("first".into()));
    let result2: Result<(), Error> = Err(Error("second".into()));

    let final_result: Result<_, Vec<Error>> = vec![result1, result2]
        .into_iter()
        .partition_result::<Vec<_>, Vec<_>, _, _>()
        .into_result();
    ```

    For `errors: Vec<E>`:
    - It checks if `errors` is empty.
    - If true, it will return `Ok(())`.
    - Otherwise, it will return `Err(errors)`.

    Since the trait is implemented for tuples of `(success: T, errors: Vec<E>)`
    and for `Vec<E>`, it works well with `partition_result` from the `itertools` crate!

    ```rust
    # use itertools::*;
    # use error_trees::*;
    struct Error(String);

    let result1: Result<(), Error> = Err(Error("first".into()));
    let result2: Result<(), Error> = Err(Error("second".into()));

    let final_result: Result<_, Vec<Error>> = vec![result1, result2]
        .into_iter()
        .partition_result::<Vec<_>, Vec<_>, _, _>()
        .into_result();
    ```
    */
    fn into_result(self) -> Result<T, E2>;
}

impl<T, E1, E2> IntoResult<T, E1, E2> for (T, Vec<E1>)
where
    Vec<E1>: Into<E2>,
{
    fn into_result(self) -> Result<T, E2> {
        let (oks, errs) = self;

        if errs.is_empty() {
            Ok(oks)
        } else {
            Err(errs.into())
        }
    }
}

impl<E1, E2> IntoResult<(), E1, E2> for Vec<E1>
where
    Vec<E1>: Into<E2>,
{
    fn into_result(self) -> Result<(), E2> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.into())
        }
    }
}

/// Convenience trait to label errors within a `Result`.
pub trait LabelResult<T, L, E> {
    /**
    Maps a label to the `ErrorTree` within the result.

    ```rust
    # use itertools::*;
    # use error_trees::*;
    struct Error(String);
    let result: Result<(), ErrorTree<&'static str, Error>> = Ok(());
    let labeled_result = result.label_error("the label");
    ```
    */
    fn label_error(self, label: L) -> Result<T, ErrorTree<L, E>>;
}

impl<T, L, E> LabelResult<T, L, E> for Result<T, E>
where
    ErrorTree<L, E>: From<E>,
{
    fn label_error(self, label: L) -> Result<T, ErrorTree<L, E>> {
        self.map_err(|e| {
            let tree: ErrorTree<L, E> = e.into();
            tree.with_label(label)
        })
    }
}

impl<T, L, E> LabelResult<T, L, E> for Result<T, ErrorTree<L, E>> {
    fn label_error(self, label: L) -> Result<T, ErrorTree<L, E>> {
        self.map_err(|tree| tree.with_label(label))
    }
}

pub trait FlattenResultErrors<T, L, E> {
    fn flatten_results(self) -> Result<T, Vec<FlatError<L, E>>>;
}

impl<T, L, E> FlattenResultErrors<T, L, E> for Result<T, ErrorTree<L, E>>
where
    L: Clone,
{
    fn flatten_results(self) -> Result<T, Vec<FlatError<L, E>>> {
        self.map_err(|tree| tree.flatten_tree())
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[derive(Debug)]
    struct Error(String);

    impl<L> From<Error> for ErrorTree<L, Error> {
        fn from(e: Error) -> Self {
            Self::leaf(e)
        }
    }

    fn faulty(error: &str) -> Result<(), Error> {
        Err(Error(error.into()))
    }

    #[test]
    fn can_build_tree_from_vec_of_results() {
        let result_1 = faulty("error1").map_err(|e| e.with_label("label1"));
        let result_2 = faulty("error2").map_err(|e| e.with_label("label2"));

        let (_, errors): (Vec<_>, Vec<_>) = vec![result_1, result_2].into_iter().partition_result();

        let tree: ErrorTree<&'static str, Error> = errors.into();
        let tree = tree.with_label("parent_label");

        let flat_errors = tree.flatten_tree();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(error1),
                    },
                    FlatError {
                        path: path2,
                        error: Error(error2),
                    },
                ]
                if path1 == &vec!["label1", "parent_label"]
                && path2 == &vec!["label2", "parent_label"]
                && error1 == "error1"
                && error2 == "error2"
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }

    #[test]
    fn can_call_into_result_from_vec_of_results() {
        let result_1 = faulty("error1").map_err(|e| e.with_label("label1"));
        let result_2 = faulty("error2").map_err(|e| e.with_label("label2"));

        let result: Result<(), ErrorTree<_, _>> = vec![result_1, result_2]
            .into_iter()
            .partition_result()
            .into_result();

        let flat_result = result.map_err(|e| e.flatten_tree());

        let flat_errors = flat_result.unwrap_err();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(error1),
                    },
                    FlatError {
                        path: path2,
                        error: Error(error2),
                    },
                ]
                if path1 == &vec!["label1"]
                && path2 == &vec!["label2"]
                && error1 == "error1"
                && error2 == "error2"
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }

    #[test]
    fn can_call_into_result_from_vec_of_errors() {
        let error1 = Error("error1".into()).with_label("label1");
        let error2 = Error("error2".into()).with_label("label2");

        let result: Result<_, ErrorTree<_, _>> = vec![error1, error2].into_result();

        let flat_result = result.map_err(|e| e.flatten_tree());

        let flat_errors = flat_result.unwrap_err();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(error1),
                    },
                    FlatError {
                        path: path2,
                        error: Error(error2),
                    },
                ]
                if path1 == &vec!["label1"]
                && path2 == &vec!["label2"]
                && error1 == "error1"
                && error2 == "error2"
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }

    // For the README
    // A function that returns an error
    fn faulty_function() -> Result<(), Error> {
        Err(Error("error".into()))
    }

    // A function that returns more than one error
    fn parent_function() -> Result<Vec<()>, ErrorTree<&'static str, Error>> {
        let result1 = faulty_function().label_error("first faulty");
        let result2 = faulty_function().label_error("second faulty");

        let result: Result<_, ErrorTree<_, _>> = vec![result1, result2]
            .into_iter()
            .partition_result::<Vec<_>, Vec<_>, _, _>()
            .into_result();
        result.label_error("parent function")
    }

    // your main function
    #[test]
    fn main_function() {
        let result = parent_function();

        let flat_results = result.flatten_results();
        let flat_errors: Vec<FlatError<&str, Error>> = flat_results.unwrap_err();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(_),
                    },
                    FlatError {
                        path: path2,
                        error: Error(_),
                    },
                ]
                if path1 == &vec!["first faulty", "parent function"]
                && path2 == &vec!["second faulty", "parent function"]
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }
}
