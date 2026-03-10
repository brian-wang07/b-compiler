/* ============================================================
   VERY LARGE B PARSER TEST (includes labels + goto)
   - Uses: globals, arrays, initializers, functions, auto/extrn,
     blocks, if/else nesting, while, switch/case/default (optional),
     labels, goto (forward/backward), calls, indexing, unary/binary,
     postfix ++/-- (remove if not supported), ternary (remove if not),
     return with/without value.
   - If your target is strict 1972 B: delete `default:` blocks,
     delete `++/--`, delete `?:`, delete `==`/relops if not present.
   ============================================================ */

/* -------- Globals -------- */
g0;
g1 1;
g2 -2;
g3 3,4,5;

A[1];
B[2] 10, 20;
C[5] 0, 1, 2, 3, 4;
D[8] 8,7,6,5,4,3,2,1;
E[16] 0,0,0,0,  1,1,1,1,  2,2,2,2,  3,3,3,3;

F[8] 1,2,3,4,5,6,7,8;
G[12] 9,8,7,6,5,4,3,2,1,0,1,2;
H[6];

flag;
tmp;

/* -------- Functions -------- */

main(argc, argv) {
    auto i, j, k, n, m, t, u, v, w, x, y, z;
    auto p, q, r, s;
    auto a, b, c, d, e;
    auto idx, sum, prod;
    auto outer, inner, limit, mode;

    i = 0;
    j = 1;
    k = 2;

    n = 10;
    m = 3;

    limit = 7;
    mode = 2;

start:
    /* unary + grouping */
    t = -n;
    u = +j;
    v = -( (n + 1) * (m + 2) );

    /* precedence stress */
    w = n + m * 2 - 4 / 2 + (3 * (2 + 1));
    x = (n + m) * (2 - 4) / (1 + 1);
    y = (n & 7) | (m ^ 3);
    z = (y ^ (x & 15)) | (n & (m + 1));

    /* arrays */
    A[0] = 11;
    B[0] = A[0] + 1;
    B[1] = B[0] * 2;

    C[0] = 5;
    C[1] = 6;
    C[2] = (B[1] - B[0]) + (A[0] * 3);
    C[3] = C[2] + 1;
    C[4] = C[3] + 1;

    idx = 0;
    sum = 0;
    prod = 1;

    /* while + nested if + goto to escape */
outer_loop_label:
    while (idx < 5) {
        sum = sum + C[idx];
        prod = prod * (C[idx] + 1);

        if (C[idx] & 1) {
            D[idx] = C[idx] + 100;
        } else {
            D[idx] = C[idx] + 200;
        }

        if (idx == 3) goto escaped;

        idx = idx + 1;
    }

escaped:
    /* nested blocks + autos */
    {
        auto t1, t2, t3, t4;
        t1 = sum;
        t2 = prod;
        t3 = (t1 + t2) / 2;
        t4 = (t3 * 3) - (t2 / 2);
        g1 = t3;
        g2 = t4;

        {
            auto inner1, inner2;
            inner1 = t4 + 1;
            inner2 = inner1 * 2;
            tmp = inner2;
        }
    }

    /* calls */
    a = add(1, 2);
    b = sub(10, 3);
    c = mul(add(1,2), sub(10,3));
    d = mix3(a, b, c);
    e = fold4(a, b, c, d);

    side_effect(a, b, c, d);

    /* if/else nesting chain */
    if (a < b) {
        r = 1;
    } else if (a == b) {
        r = 2;
    } else if (c < d) {
        r = 3;
    } else {
        r = 4;
    }

    /* label in straight-line code */
choose_mode:
    if (mode == 0) goto mode0;
    if (mode == 1) goto mode1;
    goto mode2;

mode0:
    flag = 10;
    goto after_modes;

mode1:
    flag = 20;
    goto after_modes;

mode2:
    flag = 30;

after_modes:
    /* switch/case/default (delete default blocks if unsupported) */
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
        case 4: {
            s = 400;
            g0 = s;
        }
        default: {
            s = 999;
            g0 = s;
        }
    }

    /* more array work */
    E[0] = s;
    E[1] = E[0] + 1;
    E[2] = E[1] + E[0];
    E[3] = E[2] + (E[1] * 2);
    E[4] = (E[3] & 7) | (E[2] ^ 3);
    E[5] = E[4] + (E[0] / 2);

    /* optional: postfix + ternary (remove if unsupported) */
    i++;
    j--;
    k++;
    k--;

    p = (sum ? 111 : 222);
    q = (prod ? (sum + 1) : (prod + 2));

    /* goto forward/backward resolution */
    if (g0 == 200) goto tweak;
    goto finalize;

tweak:
    g0 = g0 + 7;
    goto finalize;

finalize:
    /* second loop with backward goto */
    outer = 0;
second_loop_top:
    while (outer < limit) {
        inner = 0;

inner_loop_top:
        while (inner < 4) {
            H[inner] = (outer * 10) + inner;
            if (inner == 2) goto inner_bump;
            inner = inner + 1;
        }
        goto outer_step;

inner_bump:
        inner = inner + 1;
        goto inner_loop_top;

outer_step:
        if (outer == 3) goto early_out;
        outer = outer + 1;
        goto second_loop_top;
    }

early_out:
    g2 = g2 + outer;
    return g0;
}

/* --- helpers --- */

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

fold4(a, b, c, d) {
    auto t;
    t = (a + b) + (c + d);
    t = t + (a * d) - (b * c);
    return t;
}

side_effect(a, b, c, d) {
    auto t, u;
    auto i;
    auto s1, s2;

    t = a + b + c + d;
    u = (t & 7) | (t ^ 3);

    F[0] = t;
    F[1] = u;

    s1 = 0;
    s2 = 1;

    i = 0;

se_loop:
    while (i < 8) {
        F[i] = F[i] + i;
        s1 = s1 + F[i];
        s2 = s2 * (i + 1);

        if (i == 3) goto skip_ahead;

        i = i + 1;
    }
    goto se_end;

skip_ahead:
    i = i + 2;
    goto se_loop;

se_end:
    G[0] = s1;
    G[1] = s2;
    return;
}

noop() {
    auto x;
    x = 0;
noop_end:
    return;
}