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
        (eq? a b)))

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
    use std::assert_matches;

    use crate::{
        builtins,
        error::Result,
        eval, lexer, parser,
        runtime::{HeapObj, Runtime, Value},
    };

    use super::*;

    fn obj_eq(rt: &mut Runtime, obj0: Value, obj1: HeapObj) -> bool {
        let Value::Obj(obj0) = obj0 else {
            panic!("expected obj0 to be an ObjId")
        };
        let obj0 = Value::Obj(obj0);
        let obj1 = rt.heap.alloc_val(obj1);

        match (builtins::BuiltinType::Eq.resolve())(&[obj0, obj1], rt) {
            Ok(Value::Bool(b)) => b,
            _ => panic!("expected eq builtin to return a bool"),
        }
    }

    fn eval(rt: &mut Runtime, prog: &str) -> Result<Value> {
        let tokens = lexer::lex(prog)?;
        let ast = parser::parse(&tokens)?;

        let mut last_value = rt.heap.nil();
        for expr in ast {
            rt.gc();
            last_value = eval::eval(&expr, rt.global_env, rt)?;
        }

        Ok(last_value)
    }

    fn with_evaluated_prelude() -> Result<Runtime> {
        let mut rt = Runtime::with_print(Box::new(|s| print!("{}", s)));

        eval(&mut rt, LISPISH_PRELUDE)?;

        Ok(rt)
    }

    #[test]
    fn evaluate_prelude() -> Result<()> {
        with_evaluated_prelude().and(Ok(()))
    }

    #[test]
    fn not() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(not #t)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(not #f)"), Ok(Value::Bool(true)));

        Ok(())
    }

    #[test]
    fn and() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(and #f #f)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(and #f #t)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(and #t #f)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(and #t #t)"), Ok(Value::Bool(true)));

        Ok(())
    }

    #[test]
    fn or() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(or #f #f)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(or #f #t)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(or #t #f)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(or #t #t)"), Ok(Value::Bool(true)));

        Ok(())
    }

    #[test]
    fn xor() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(xor #f #f)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(xor #f #t)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(xor #t #f)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(xor #t #t)"), Ok(Value::Bool(false)));

        Ok(())
    }

    #[test]
    fn eq() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(= 1 1)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(= 1 0)"), Ok(Value::Bool(false)));

        Ok(())
    }

    #[test]
    fn gt() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(> 1 0)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(> 0 1)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(> 0 0)"), Ok(Value::Bool(false)));

        Ok(())
    }

    #[test]
    fn geq() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(>= 1 0)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(>= 0 1)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(>= 0 0)"), Ok(Value::Bool(true)));

        Ok(())
    }

    #[test]
    fn leq() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(<= 1 0)"), Ok(Value::Bool(false)));
        assert_matches!(eval(&mut rt, "(<= 0 1)"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(<= 0 0)"), Ok(Value::Bool(true)));

        Ok(())
    }

    #[test]
    fn is_null() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(eval(&mut rt, "(null? '())"), Ok(Value::Bool(true)));
        assert_matches!(eval(&mut rt, "(null? '(1 2 3))"), Ok(Value::Bool(false)));

        Ok(())
    }

    #[test]
    fn map() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        let res = eval(&mut rt, "(map (lambda (n) (+ n 1)) '(1 2 3))")?;
        assert!(obj_eq(
            &mut rt,
            res,
            HeapObj::List(vec![
                Value::Number(2.),
                Value::Number(3.),
                Value::Number(4.)
            ])
        ));

        Ok(())
    }

    #[test]
    fn filter() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        let res = eval(&mut rt, "(filter (lambda (n) (> n 0)) '(-1 0 1))")?;
        assert!(obj_eq(&mut rt, res, HeapObj::List(vec![Value::Number(1.)])));

        Ok(())
    }

    #[test]
    fn any() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(
            eval(&mut rt, "(any (lambda (n) (> n 0)) '(-1 0 1))"),
            Ok(Value::Bool(true))
        );
        assert_matches!(
            eval(&mut rt, "(any (lambda (n) (> n 0)) '(-1 -2 -3))"),
            Ok(Value::Bool(false))
        );

        Ok(())
    }

    #[test]
    fn all() -> Result<()> {
        let mut rt = with_evaluated_prelude()?;

        assert_matches!(
            eval(&mut rt, "(all (lambda (n) (< n 0)) '(-1 -2 -3))"),
            Ok(Value::Bool(true))
        );
        assert_matches!(
            eval(&mut rt, "(all (lambda (n) (> n 0)) '(-1 0 1))"),
            Ok(Value::Bool(false))
        );

        Ok(())
    }
}
