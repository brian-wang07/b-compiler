/* ===========================
   BIG B PARSER TEST v2
   Includes: labels + goto, plus everything from before.
   NOTE: if your historic-B switch has no `default`, delete the `default:` blocks.
   =========================== */

/* -------- Globals -------- */
g0;
g1 1;
g2 -2;

A[1];
B[2] 10, 20;
C[5] 0, 1, 2, 3, 4;
D[8] 8,7,6,5,4,3,2,1;
E[16] 0,0,0,0,  1,1,1,1,  2,2,2,2,  3,3,3,3;

F[4];
G[6] 9, 8, 7, 6, 5, 4;

/* -------- Functions -------- */

main(argc, argv) {
    extrn a, b, c;
    auto i, j, k, n, m, t, u, v, w, x, y, z;
    auto p, q, r, s;
    auto a, b, c, d;
    auto idx, sum, prod;

    /* basic assigns */
    i = 0;
    j = 1;
    k = 2;
    n = 10;
    m = 3;

    /* label + straight-line flow */
entry:
    t = -n;
    u = +j;
    v = -( (n + 1) * (m + 2) );

    /* precedence stress */
    w = n + m * 2 - 4 / 2 + (3 * (2 + 1));
    x = (n + m) * (2 - 4) / (1 + 1);
    y = (n & 7) | (m ^ 3);

    /* array indexing */
    A[0] = 11;
    B[0] = A[0] + 1;
    B[1] = B[0] * 2;
    C[2] = (B[1] - B[0]) + (A[0] * 3);

    idx = 0;
    sum = 0;
    prod = 1;

    /* while loop + nested if + goto out of nested control */
loop_top:
    while (idx < 5) {
        sum = sum + C[idx];
        prod = prod * (C[idx] + 1);

        if (C[idx] & 1) {
            D[idx] = C[idx] + 100;
        } else {
            D[idx] = C[idx] + 200;
        }

        /* force a goto jump when idx hits 3 */
        if (idx == 3) goto after_loop;

        idx = idx + 1;
    }

after_loop:
    /* nested blocks + autos */
    {
        auto t1, t2, t3;
        t1 = sum;
        t2 = prod;
        t3 = (t1 + t2) / 2;
        g1 = t3;
        {
            auto inner;
            inner = t3 * 2 + 1;
            g2 = inner;
        }
    }

    /* calls: simple + nested */
    a = add(1, 2);
    b = sub(10, 3);
    c = mul(add(1,2), sub(10,3));
    d = mix3(a, b, c);

    side_effect(a, b, c, d);

    /* multi-branch if/else-if/else (else-if is nesting) */
    if (a < b) {
        r = 1;
    } else if (a == b) {
        r = 2;
    } else {
        r = 3;
    }

    /* switch / case / default (delete default blocks if not supported) */
    switch (r) {
        case 1: {
            s = 100;
            g0 = s;
        }
        case 2: {
            s = 200;
            g0 = s;
        }
        case 3: {
            s = 300;
            g0 = s;
        }
        default: {
            s = 999;
            g0 = s;
        }
    }

    /* more indexing and expression statements */
    E[0] = s;
    E[1] = E[0] + 1;
    E[2] = E[1] + E[0];
    E[3] = E[2] + (E[1] * 2);

    /* goto forward, then back, to test label resolution */
    if (g0 == 200) goto tweak;
    goto done;

tweak:
    g0 = g0 + 7;
    goto done;

done:
    return g0;
}

add(a, b) {
    auto t;
    t = a + b;
    return t;
}

sub(a, b) {
    auto t;
    t = a - b;
    return t;
}

mul(a, b) {
    auto t;
    t = a * b;
    return t;
}

mix3(a, b, c) {
    auto t;
    t = a + (b * 2) + (c * 3);
    return t;
}

side_effect(a, b, c, d) {
    auto t, u;
    auto i;

    t = a + b + c + d;
    u = (t & 7) | (t ^ 3);
    F[0] = t;
    F[1] = u;

    i = 0;

seloop:
    while (i < 4) {
        G[i] = F[i] + i;

        /* test backward goto from inside loop */
        if (i == 2) goto bump;

        i = i + 1;
    }
    goto seend;

bump:
    i = i + 1;
    goto seloop;

seend:
    return;
}

/* Small function to test empty-ish structure + label */
noop() {
    auto x;
    x = 0;
noop_end:
    return;
}