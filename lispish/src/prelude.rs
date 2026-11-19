pub const LISPISH_PRELUDE: &str = /* lisp */
    r##"
    ; logic
    (define not
      (lambda (x)
        (if x #f #t)))

    (define and
      (lambda (a b)
        (if a b #f)))

    (define or
      (lambda (a b)
        (if a #t b)))

    (define xor
      (lambda (a b)
        (and (or a b)
             (not (and a b)))))

    ; cmp
    (define =
      (lambda (a b)
        (eq a b)))

    (define >
      (lambda (a b)
        (< b a)))

    (define >=
      (lambda (a b)
        (not (< a b))))

    (define <=
      (lambda (a b)
        (not (< b a))))

    (define null?
      (lambda (x)
        (eq? x '())))

    ; functional
    (define map
      (lambda (fn c)
        (if (null? c)
            '()
            (cons (fn (car c))
                  (map fn (cdr c))))))

    (define filter
      (lambda (pred c)
        (if (null? c)
            c
            (if (pred (car c))
                (cons (car c)
                      (filter pred (cdr c)))
                (filter pred (cdr c))))))

    (define any
      (lambda (pred c)
        (if (null? c)
            #f
            (if (pred (car c))
                #t
                (any pred (cdr c))))))

    (define all
      (lambda (pred c)
        (if (null? c)
            #t
            (if (pred (car c))
                (all pred (cdr c))
                #f))))
"##;

#[cfg(test)]
mod test {
    use crate::{error::Result, eval, lexer, parser, runtime::Runtime};

    use super::*;

    #[test]
    fn evaluate_prelude() -> Result<()> {
        let mut rt = Runtime::with_print(Box::new(|s| print!("{}", s)));

        let tokens = lexer::lex(LISPISH_PRELUDE)?;
        let ast = parser::parse(&tokens)?;

        for expr in ast {
            eval::eval(&expr, rt.global_env, &mut rt)?;
            rt.gc();
        }

        Ok(())
    }
}
