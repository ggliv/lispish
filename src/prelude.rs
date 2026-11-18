pub const LISPISH_PRELUDE: &str = /* lisp */
    r##"
    ; logic
    (define not (lambda (x) (if x #f #t)))
    (define and (lambda (a b) (if a b #f)))
    (define or (lambda (a b) (if a #t b)))
    (define xor (lambda (a b) (and (or a b) (not (and a b)))))
    ; cmp
    (define = (lambda (a b) (eq a b)))
    (define > (lambda (a b) (< b a)))
    (define >= (lambda (a b) (not (< a b))))
    (define <= (lambda (a b) (not (< b a))))
    (define null? (lambda (x) (eq? x '())))
    ; functional
    ; TODO map filter any all
"##;

#[cfg(test)]
mod test {
    use crate::{
        builtins::init_global_env,
        error::Result,
        eval::{self, Heap},
        lexer, parser,
    };

    use super::*;

    #[test]
    fn evaluate_prelude() -> Result<()> {
        let mut heap = Heap::default();
        let global_env = init_global_env(&mut heap);

        let tokens = lexer::lex(LISPISH_PRELUDE)?;
        let ast = parser::parse(&tokens)?;

        for expr in ast {
            eval::eval(&expr, global_env, &mut heap)?;
            heap.gc([global_env]);
        }

        Ok(())
    }
}
