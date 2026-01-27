/* 
   B Language Stress Test
   Testing: Octal, String Escapes, Compound Assignments, and 8-char Significance
*/

main() {
    auto a, b, c[10], longvariable_name;
    extrn putchar, x, y;

    /* 1. Numeric Stress: Decimal vs Octal */
    a = 123;
    b = 0177;    /* Octal 177 = Decimal 127 */
    x = 0005;    /* Multiple leading zeros */
    y = 0;       /* Is it decimal or octal? (B treats it as octal base) */

    /* 2. String & Char Stress: The Asterisk (*) Escape */
    longvariable_name = "Line 1*nLine 2*tTabbed*eEOF";
    a = "Quotes: *' and ** asterisk";
    b = '*n';    /* Character constant newline */
    c[0] = '*0';  /* Null character */

    /* 3. Operator Ambiguity Stress */
    a =+ 5;      /* Compound Assign (Add) */
    b =- 10;     /* Compound Assign (Sub) */
    x =* y;      /* Compound Assign (Mul) - Note the asterisk! */
    a =<< 2;     /* Compound Assign (Left Shift) */
    
    if (a == b) {
        a++;     /* Increment */
        b--;     /* Decrement */
    }

    if (a != b & c[1] <= 10) {
        x = a >> 1 | b << 2; /* Bitwise and Shifts */
    }

    /* 4. Comment & Slash Stress */
    a = b / 5;   /* Division */
    a = b / * comment * / 5; /* Division with comment in between */

    /* 5. The "Ternary" and Logical */
    x = (a < b) ? a : b;
    y = !a | ~b ^ x;

    return (x == y);
}

/* Global definitions with initializers */
x 100;
y 0144;
s "Global String**";
x = 'a';
y = 'ab'
z = '';