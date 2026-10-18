//! The golden graphs, built again with `SynthDefBuilder::add`, and with
//! `unary_op` and `binary_op`: the crate writes the bytes that sclang 3.14.1
//! wrote.

mod common;

#[test]
fn every_golden_graph_built_with_add_writes_the_bytes_of_sclang() {
    for name in common::NAMES {
        assert_eq!(common::written(name), common::golden(name), "{name}");
    }
}

#[test]
fn the_operators_by_name_write_the_bytes_of_sclang() {
    let def = common::operators_by_name();
    assert_eq!(def.as_bytes().unwrap(), common::golden("operators"));
}
