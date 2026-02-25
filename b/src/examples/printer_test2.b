
printf(fmt, x1,x2,x3,x4,x5,x6,x7,x8,x9) {
	auto adx, x, c, i, j;
	extrn printn, char, putchar;

	i= 0;	/* fmt index */
	adx = &x1;	/* argument pointer */
loop :
	while((c=char(fmt,i++) )) {
		if(c == '*e')
			return;
		putchar(c);
	}
	x = *adx++;
	switch c = char(fmt,i++) {

	case 'd': /* decimal */
	case 'o': /* octal */
		if(x < O) {
			x = -x ;
			putchar('-');
		}
		goto loop;

	case 'c' : /* char */
		putchar(x);
		goto loop;

	case 's': /* string */
		while(c=char(x, j++)) != '*e')
			putchar(c);
		goto loop;
	}
	putchar('%') ;
	i--;
	adx--;
	goto loop;
}