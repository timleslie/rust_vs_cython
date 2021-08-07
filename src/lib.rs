#![allow(non_snake_case)]
use numpy::{Ix1, PyArray, PyReadonlyArray};
use pyo3::prelude::*;
use std::convert::TryInto;

#[pyfunction]
fn scc(
    idxptr: PyReadonlyArray<i32, Ix1>,
    indices: PyReadonlyArray<i32, Ix1>,
) -> PyResult<(i32, Py<PyArray<i32, Ix1>>)> {
    let idxptr = idxptr.as_slice()?;
    let indices = indices.as_slice()?;
    let N: i32 = idxptr.len().try_into().unwrap();
    let END = N + 1;
    let VOID = N + 2;
    let mut stack_f = vec![VOID; idxptr.len()];
    let mut stack_b = vec![VOID; idxptr.len()];

    let mut label = N;
    let mut index = 0;
    let mut ss_head = END;

    pyo3::Python::with_gil(|py| {
        let mut _lowlinks: &PyArray<i32, Ix1> =
            PyArray::from_vec(py, vec![VOID; N.try_into().unwrap()]);
        let mut lowlinks = unsafe { _lowlinks.as_slice_mut()? };
        // Iterate over every node in order
        for v1 in 0..N {
            let v1i: usize = v1.try_into().unwrap();
            // If not node hasn't been processed yet, it won't have a lowlink or a label
            if lowlinks[v1i] == VOID {
                // DFS-stack push;
                // At this point, the DFS stack is empty, so pushing sets both the
                // forward and backwards pointers to end.
                let mut stack_head = v1;
                stack_f[v1i] = END;
                stack_b[v1i] = END;
                // We'll now proceed wih the inner loop algorithm until the stack is empty
                while stack_head != END {
                    let v = stack_head;
                    let vi: usize = v.try_into().unwrap();
                    if lowlinks[vi] == VOID {
                        // If the top node in the stack hasn't been visited yet,
                        // assign it the next index value.
                        lowlinks[vi] = index;
                        index += 1;

                        // Visit all of the nodes accessible from v and push then onto the stack
                        // ahead of v. If they're already in the stack, bring them to the top.
                        let range_end = if v == N - 1 {
                            indices.len()
                        } else {
                            idxptr[vi + 1].try_into().unwrap()
                        };

                        for &w in &indices[idxptr[vi].try_into().unwrap()..range_end] {
                            let wi: usize = w.try_into().unwrap();
                            if lowlinks[wi] == VOID {
                                if stack_f[wi] != VOID {
                                    // w is already inside the stack, so excise it.
                                    let f = stack_f[wi];
                                    let b = stack_b[wi];
                                    if b != END {
                                        let bi: usize = b.try_into().unwrap();
                                        stack_f[bi] = f;
                                    }
                                    if f != END {
                                        let fi: usize = f.try_into().unwrap();
                                        stack_b[fi] = b;
                                    }
                                }
                                // Add w to the top of the stack. end <-> w <-> stack_head <-> ...
                                stack_f[wi] = stack_head;
                                stack_b[wi] = END;
                                let stack_head_i: usize = stack_head.try_into().unwrap();
                                stack_b[stack_head_i] = w;
                                stack_head = w;
                            }
                        }
                    } else {
                        // DFS-stack pop
                        stack_head = stack_f[vi];
                        // If the stack_head isn't the end
                        if stack_head < N {
                            let stack_head_i: usize = stack_head.try_into().unwrap();
                            stack_b[stack_head_i] = END;
                        }
                        stack_f[vi] = VOID;
                        stack_b[vi] = VOID;

                        // Find out whether this node is a root node
                        // We look at all its linked nodes (which have now all had this
                        // process applied to them!). If none of them have a lower index than this
                        // node then we have a root value. Otherwise we reset the index to the lowest
                        // index.
                        let mut root = true;
                        let mut low_v = lowlinks[vi];
                        let range_end = if v == N - 1 {
                            indices.len()
                        } else {
                            idxptr[vi + 1].try_into().unwrap()
                        };
                        for &w in &indices[idxptr[vi].try_into().unwrap()..range_end] {
                            let wi: usize = w.try_into().unwrap();
                            let low_w = lowlinks[wi];
                            if low_w < low_v {
                                low_v = low_w;
                                root = false;
                            }
                        }
                        lowlinks[vi] = low_v;

                        let ss = &mut stack_f;
                        if root {
                            // Found a root node. This means we've found the root of
                            // a strongly connected component. All the items on the stack
                            // with an index greater or equal to the current nodes index
                            // are part of the SCC and get the same level.
                            // We can reclaim their index values at this point.

                            // while S not empty and rindex[v] <= rindex[top[S]]
                            let mut ss_head_i: usize = ss_head.try_into().unwrap();
                            while ss_head != END && lowlinks[vi] <= lowlinks[ss_head_i] {
                                let w = ss_head; // w = pop(S)
                                let wi: usize = w.try_into().unwrap();
                                ss_head = ss[wi];
                                ss_head_i = ss_head.try_into().unwrap();
                                ss[wi] = VOID;
                                let labels = &mut lowlinks;
                                labels[wi] = label; // rindex[w] = c;
                                index -= 1; // index = index - 1
                            }
                            let labels = &mut lowlinks;
                            if index > 0 {
                                index -= 1;
                            }
                            labels[vi] = label; // rindex[v] = c
                            // Move to the next available label value
                            label -= 1;
                        } else {
                            // We haven't got to the root of this group, so add v to the sets stack
                            ss[vi] = ss_head; // push(S, v)
                            ss_head = v;
                        }
                    }
                }
            }
        }

        let labels = &mut lowlinks;

        for label in labels.iter_mut() {
            *label = N - *label
        }
        Ok((N - label, _lowlinks.to_owned()))
    })
}

#[pymodule]
fn rust_scc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(scc, m)?)?;

    Ok(())
}
