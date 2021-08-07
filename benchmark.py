import random
import numpy as np
from rust_scc import scc
from scipy.sparse.csgraph import connected_components
from scipy.sparse import csr_matrix


def run_scc(m):
    indptr = m.indptr[:-1]
    indices = m.indices
    return rust(indptr, indices)


@profile
def rust(indptr, indices):
    return scc(indptr, indices)


@profile
def python(m):
    return connected_components(m, connection="strong")


def generate_sparse(N, k):
    """
    Generate an directed graph as an NxN sparse matrix where each node has
    [0, k) randomly connected edges (including possible self connections).

    The resulting graph will have V=N and E ~= N*k/2
    """
    indices = []
    idxptr = [0]
    i = 0
    for _ in range(N):
        _k = random.randrange(k)
        indices += sorted(random.sample(range(N), _k))
        i += _k
        idxptr.append(i)
    return csr_matrix(([1] * len(indices), indices, idxptr), shape=(N, N))


def debug_differences(m, x, y, msg):
    """
    If the algorithms disagree lets dump the info we need to reproduce the error
    """
    print(x)
    print(y)
    print(m.indices)
    print(m.indptr)
    np.save("indices", m.indices)
    np.save("indptr", m.indptr)
    raise Exception(msg)


def main(N, k):
    m = generate_sparse(N, k)
    x = run_scc(m)
    y = python(m)
    # Verify that the two algorithms produce identical results
    if x[0] != y[0]:
        debug_differences(m, x, y, "Different counts")
    if not (x[1] == y[1]).all():
        debug_differences(m, x, y, "Different arrays")


# N and k are injected into our namespace by the benchmark runner
main(N, k)
