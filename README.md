# Error Trees

Fail in a spectacular manner with multiple errors, instead only a single one!

```rust
// Your error type
#[derive(debug)]
struct Error(String);

// A function that returns an error
fn faulty_function() -> Result<(), Error> {
    Err(Error("error".into()))
}

// A function that returns more than one error
fn parent_function() -> Result<((), ()), ErrorTree<&'static str, Error>> {
    let result1 = faulty_function().label_error("first faulty");
    let result2 = faulty_function().label_error("second faulty");

    vec![result1, result2].into_iter()
        .partition_result()
        .into_result()
        .label_error("parent function")
}

// your main function
fn main_function() {
    let result = parent_function();

    let flat_results = result.flatten_results();
    let flat_errors : Vec<FlatError<&str, Error>>> = tree.unwrap_err();

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
```
