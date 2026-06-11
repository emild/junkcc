use std::collections::HashMap;

pub trait Precedence {
    fn precedence(&self) -> u32;
}


#[derive(Debug, Clone)]
pub enum TypedExpression {
    TypedExp(Option<Type>, Expression)
}


#[derive(Debug, Clone)]
pub enum Expression {
    Constant(Const),
    Var(String),
    Cast(Type /* target_type */, Box<TypedExpression>),
    Unary(UnaryOperator, Box<TypedExpression>),
    PreIncrement(Box<TypedExpression>),
    PreDecrement(Box<TypedExpression>),
    PostIncrement(Box<TypedExpression>),
    PostDecrement(Box<TypedExpression>),
    Binary(BinaryOperator, Box<TypedExpression>, Box<TypedExpression>),
    Assignment(Box<TypedExpression>, Box<TypedExpression>),
    CompoundAssignment(BinaryOperator, Box<TypedExpression>, Box<TypedExpression>),
    Conditional(Box<TypedExpression> /* condition */, Box<TypedExpression> /* true */, Box<TypedExpression> /* false */),
    FunctionCall(String /* func_name */, Vec<TypedExpression> /* args */)
}



pub fn typex_get_type(typed_expr: &TypedExpression) -> Type
{
    let TypedExpression::TypedExp(typ, _expr) = typed_expr;

    typ.clone().unwrap()
}


pub fn typex_set_type(expr: Expression, typ: Type) -> TypedExpression
{
    TypedExpression::TypedExp(Some(typ), expr)
}


pub fn typex_init(expr: Expression) -> TypedExpression
{
    TypedExpression::TypedExp(None, expr)
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntegerConst {
    ConstInt(i32),
    ConstUInt(u32),
    ConstLong(i64),
    ConstULong(u64)
}

impl IntegerConst {
    pub fn to_i64(&self) -> i64
    {
        match self {
            Self::ConstInt(c) => *c as i64,
            Self::ConstUInt(c) => *c as i64,
            Self::ConstLong(c) => *c,
            Self::ConstULong(c) => *c as i64
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum FloatingPointConst {
    ConstDouble(f64)
}


#[derive(Debug, Clone, PartialEq/*, Eq, Hash*/)]
pub enum Const {
    I(IntegerConst),
    F(FloatingPointConst)
}


impl From<i32> for Const {
    fn from(val: i32) -> Const {
        Const::I(IntegerConst::ConstInt(val))
    }
}

impl From<i64> for Const {
    fn from(val: i64) -> Const {
        Const::I(IntegerConst::ConstLong(val))
    }
}

impl From<u32> for Const {
    fn from(val: u32) -> Const {
        Const::I(IntegerConst::ConstUInt(val))
    }
}

impl From<u64> for Const {
    fn from(val: u64) -> Const {
        Const::I(IntegerConst::ConstULong(val))
    }
}

impl From<f64> for Const {
    fn from(val: f64) -> Const {
        Const::F(FloatingPointConst::ConstDouble(val))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ValueType {
    SignedInt,
    UnsignedInt,
    FloatingPoint
}

#[derive(Clone)]
struct ConstIntermediateFormat {
    pub value_type: ValueType,
    pub i_val: Option<i64>,
    pub u_val: Option<u64>,
    pub f_val: Option<f64>
}


impl ConstIntermediateFormat
{
    pub fn to_const(&self, typ: &Type) -> Const
    {
        match self.value_type {
            ValueType::SignedInt        => Const::from_i64(self.i_val.unwrap(), typ),
            ValueType::UnsignedInt      => Const::from_u64(self.u_val.unwrap(), typ),
            ValueType::FloatingPoint    => Const::from_f64(self.f_val.unwrap(), typ),
        }
    }
}


impl From<i32> for ConstIntermediateFormat {
    fn from(i_val: i32) -> ConstIntermediateFormat
    {
        ConstIntermediateFormat::from(i_val as i64)
    }
}


impl From<i64> for ConstIntermediateFormat {
    fn from(i_val: i64) -> ConstIntermediateFormat
    {
        ConstIntermediateFormat {
            value_type: ValueType::SignedInt,
            i_val: Some(i_val),
            u_val: None,
            f_val: None
        }
    }
}

impl From<u32> for ConstIntermediateFormat {
    fn from(u_val: u32) -> ConstIntermediateFormat
    {
        ConstIntermediateFormat::from(u_val as u64)
    }
}


impl From<u64> for ConstIntermediateFormat {
    fn from(u_val: u64) -> ConstIntermediateFormat
    {
        ConstIntermediateFormat {
            value_type: ValueType::UnsignedInt,
            i_val: None,
            u_val: Some(u_val),
            f_val: None
        }
    }
}

impl From<f64> for ConstIntermediateFormat {
    fn from(f_val: f64) -> ConstIntermediateFormat
    {
        ConstIntermediateFormat {
            value_type: ValueType::FloatingPoint,
            i_val: None,
            u_val: None,
            f_val: Some(f_val)
        }
    }
}


impl From<&Const> for ConstIntermediateFormat {
    fn from(c: &Const) -> ConstIntermediateFormat {
        match c {
            Const::I(IntegerConst::ConstInt(c)) => ConstIntermediateFormat::from(*c),
            Const::I(IntegerConst::ConstLong(c)) => ConstIntermediateFormat::from(*c),
            Const::I(IntegerConst::ConstUInt(c)) => ConstIntermediateFormat::from(*c),
            Const::I(IntegerConst::ConstULong(c)) => ConstIntermediateFormat::from(*c),
            Const::F(FloatingPointConst::ConstDouble(c)) => ConstIntermediateFormat::from(*c)
        }
    }
}



fn get_const_common_type(a: &Const, b: &Const) -> Type
{
    let typ_a = a.get_type();
    let typ_b = b.get_type();

    let common_typ = super::get_common_type(&typ_a, &typ_b);

    common_typ
}




impl Const {

    fn from_i64(i_val: i64, typ: &Type) -> Self
    {
        match typ {
            Type::Int       => Const::I(IntegerConst::ConstInt(i_val as i32)),
            Type::Long      => Const::I(IntegerConst::ConstLong(i_val)),
            Type::UInt      => Const::I(IntegerConst::ConstUInt(i_val as u32)),
            Type::ULong     => Const::I(IntegerConst::ConstULong(i_val as u64)),
            Type::Double    => Const::F(FloatingPointConst::ConstDouble(i_val as f64)),

            _ => { panic!("Attempt to convert from type: '{:?}'", typ); }
        }
    }


    fn from_u64(u_val: u64, typ: &Type) -> Self
    {
        match typ {
            Type::Int       => Const::I(IntegerConst::ConstInt(u_val as i32)),
            Type::Long      => Const::I(IntegerConst::ConstLong(u_val as i64)),
            Type::UInt      => Const::I(IntegerConst::ConstUInt(u_val as u32)),
            Type::ULong     => Const::I(IntegerConst::ConstULong(u_val)),
            Type::Double    => Const::F(FloatingPointConst::ConstDouble(u_val as f64)),

            _ => { panic!("Attempt to convert from type: '{:?}'", typ); }
        }
    }


    fn from_f64(f_val: f64, typ: &Type) -> Self
    {
        match typ {
            Type::Int       => Const::I(IntegerConst::ConstInt(f_val as i32)),
            Type::Long      => Const::I(IntegerConst::ConstLong(f_val as i64)),
            Type::UInt      => Const::I(IntegerConst::ConstUInt(f_val as u32)),
            Type::ULong     => Const::I(IntegerConst::ConstULong(f_val as u64)),
            Type::Double    => Const::F(FloatingPointConst::ConstDouble(f_val)),

            _ => { panic!("Attempt to convert from type: '{:?}'", typ); }
        }
    }


    pub fn to_typex(&self) -> TypedExpression
    {
        let typ = self.get_type();
        typex_set_type(Expression::Constant(self.clone()), typ)
    }

    pub fn get_type(&self) -> Type
    {
        match self {
            Const::I(IntegerConst::ConstInt(_))             => Type::Int,
            Const::I(IntegerConst::ConstUInt(_))            => Type::UInt,
            Const::I(IntegerConst::ConstLong(_))            => Type::Long,
            Const::I(IntegerConst::ConstULong(_))           => Type::ULong,
            Const::F(FloatingPointConst::ConstDouble(_))    => Type::Double
        }
    }


    pub fn is_floating_point(&self) -> bool
    {
        match self {
            Const::F(_)
                => true,
            _
                => false
        }
    }



    pub fn to_i64(&self) -> i64
    {
        let intrm = ConstIntermediateFormat::from(self);

        match intrm.value_type {
            ValueType::SignedInt => intrm.i_val.unwrap(),
            ValueType::UnsignedInt => intrm.u_val.unwrap() as i64,
            ValueType::FloatingPoint => { panic!("Attempt to convert floating point to i64"); }
        }
    }

    pub fn to_string(&self) -> String
    {
        let intrm = ConstIntermediateFormat::from(self);

        match intrm.value_type {
            ValueType::SignedInt => format!("{}",intrm.i_val.unwrap()),
            ValueType::UnsignedInt => format!("{}",intrm.u_val.unwrap()),
            ValueType::FloatingPoint => format!("{}",intrm.f_val.unwrap())
        }
    }


    pub fn is_false(&self) -> bool
    {
        match self {
            Const::I(IntegerConst::ConstInt(0))      |
            Const::I(IntegerConst::ConstUInt(0))     |
            Const::I(IntegerConst::ConstLong(0))     |
            Const::I(IntegerConst::ConstULong(0))    |
            Const::F(FloatingPointConst::ConstDouble(0.0))
                => true,

            _   => false
        }
    }

    pub fn is_true(&self) -> bool
    {
        !self.is_false()
    }


    pub fn convert_to(&self, typ: &Type) -> Const
    {
        match (self, typ) {
            //micro-optimization
            (Const::I(IntegerConst::ConstInt(_)),           Type::Int)    |
            (Const::I(IntegerConst::ConstUInt(_)),          Type::UInt)   |
            (Const::I(IntegerConst::ConstLong(_)),          Type::Long)   |
            (Const::I(IntegerConst::ConstULong(_)),         Type::ULong)  |
            (Const::F(FloatingPointConst::ConstDouble(_)),  Type::Double)
                => self.clone(),
            _ => {
                let intrm = ConstIntermediateFormat::from(self);
                intrm.to_const(&typ)
            }
        }
    }


    pub fn complement(&self) -> Const
    {
        match self {
            Const::I(IntegerConst::ConstInt(c))     => Const::I(IntegerConst::ConstInt(!c)),
            Const::I(IntegerConst::ConstUInt(c))    => Const::I(IntegerConst::ConstUInt(!c)),
            Const::I(IntegerConst::ConstLong(c))    => Const::I(IntegerConst::ConstLong(!c)),
            Const::I(IntegerConst::ConstULong(c))   => Const::I(IntegerConst::ConstULong(!c)),
            Const::F(_) => { panic!("~ cannot be applied to floating point"); }
        }
    }

    pub fn logical_not(&self) -> Const
    {
        match self {
            Const::I(IntegerConst::ConstInt(c))     => Const::I(IntegerConst::ConstInt((*c == 0) as i32)),
            Const::I(IntegerConst::ConstUInt(c))    => Const::I(IntegerConst::ConstInt((*c == 0) as i32)),
            Const::I(IntegerConst::ConstLong(c))    => Const::I(IntegerConst::ConstInt((*c == 0) as i32)),
            Const::I(IntegerConst::ConstULong(c))   => Const::I(IntegerConst::ConstInt((*c == 0) as i32)),
            Const::F(FloatingPointConst::ConstDouble(c)) =>
                Const::I(IntegerConst::ConstInt(
                    if *c == 0.0 {
                        1
                    }
                    else {
                        0
                    }
                )
            )
        }
    }


    pub fn unary_minus(&self) -> Const
    {
        match self {
            Const::I(IntegerConst::ConstInt(c))  => Const::I(IntegerConst::ConstInt(-*c)),
            Const::I(IntegerConst::ConstUInt(c))  => Const::I(IntegerConst::ConstUInt((-(*c as i32)) as u32)),
            Const::I(IntegerConst::ConstLong(c))  => Const::I(IntegerConst::ConstLong(-*c)) ,
            Const::I(IntegerConst::ConstULong(c))  => Const::I(IntegerConst::ConstULong((-(*c as i64)) as u64)),
            Const::F(FloatingPointConst::ConstDouble(c)) => Const::F(FloatingPointConst::ConstDouble(-*c))
        }
    }


    pub fn unary_plus(&self) -> Const
    {
        self.clone()
    }


    pub fn add(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a + *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a + *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a + *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a + *b)),
            (Const::F(FloatingPointConst::ConstDouble(a)), Const::F(FloatingPointConst::ConstDouble(b))) => Const::F(FloatingPointConst::ConstDouble(*a + *b)),

            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() += intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() += intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => *intrm_r.f_val.as_mut().unwrap() += intrm_b.f_val.unwrap()
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }





    pub fn sub(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a - *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a - *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a - *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a - *b)),
            (Const::F(FloatingPointConst::ConstDouble(a)), Const::F(FloatingPointConst::ConstDouble(b))) => Const::F(FloatingPointConst::ConstDouble(*a - *b)),

            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() -= intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() -= intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => *intrm_r.f_val.as_mut().unwrap() -= intrm_b.f_val.unwrap()
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }


    pub fn mul(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a * *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a * *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a * *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a * *b)),
            (Const::F(FloatingPointConst::ConstDouble(a)), Const::F(FloatingPointConst::ConstDouble(b))) => Const::F(FloatingPointConst::ConstDouble(*a * *b)),

            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() *= intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() *= intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => *intrm_r.f_val.as_mut().unwrap() *= intrm_b.f_val.unwrap()
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }



    pub fn div(&self, other: &Const) -> Const
    {
        assert!(other.is_floating_point() || other.is_true() );
        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a / *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a / *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a / *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a / *b)),
            (Const::F(FloatingPointConst::ConstDouble(a)), Const::F(FloatingPointConst::ConstDouble(b))) => Const::F(FloatingPointConst::ConstDouble(*a / *b)),

            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() /= intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() /= intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => *intrm_r.f_val.as_mut().unwrap() /= intrm_b.f_val.unwrap()
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }



    pub fn modulo(&self, other: &Const) -> Const
    {
        assert!(!self.is_floating_point());
        assert!(!other.is_floating_point());
        assert!(other.is_true());
        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a % *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a % *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a % *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a % *b)),
            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() %= intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() %= intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => { panic!("Cannot apply '%' to floating point"); }
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }




    pub fn bin_and(&self, other: &Const) -> Const
    {
        assert!(!self.is_floating_point());
        assert!(!other.is_floating_point());

        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a & *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a & *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a & *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a & *b)),
            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() &= intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() &= intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => { panic!("Cannot apply '&' to floating point"); }
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }


    pub fn bin_or(&self, other: &Const) -> Const
    {
        assert!(!self.is_floating_point());
        assert!(!other.is_floating_point());

        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a | *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a | *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a | *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a | *b)),
            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() |= intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() |= intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => { panic!("Cannot apply '|' to floating point"); }
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }


    pub fn bin_xor(&self, other: &Const) -> Const
    {
        assert!(!self.is_floating_point());
        assert!(!other.is_floating_point());

        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt(*a ^ *b)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstUInt(*a ^ *b)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstLong(*a ^ *b)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstULong(*a ^ *b)),
            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() ^= intrm_b.i_val.unwrap(),
                    ValueType::UnsignedInt      => *intrm_r.u_val.as_mut().unwrap() ^= intrm_b.u_val.unwrap(),
                    ValueType::FloatingPoint    => { panic!("Cannot apply '^' to floating point"); }
                };

                intrm_r.to_const(&common_typ)
            }
        }
    }


    pub fn left_shift(&self, other: &Const) -> Const
    {
        assert!(!self.is_floating_point());
        assert!(!other.is_floating_point());

        let typ = self.get_type();
        let intrm_self = ConstIntermediateFormat::from(self);
        let other = other.convert_to(&Type::Int);
        let intrm_other = ConstIntermediateFormat::from(&other);

        let mut result = intrm_self.clone();

        match intrm_self.value_type {
            ValueType::SignedInt => { result.i_val = Some(intrm_self.i_val.unwrap() << intrm_other.i_val.unwrap()); },
            ValueType::UnsignedInt => { result.u_val = Some(intrm_self.u_val.unwrap() << intrm_other.i_val.unwrap()); },
            ValueType::FloatingPoint => { panic!("<< Cannot be applied to floating point"); }
        }

        result.to_const(&typ)
    }



    pub fn right_shift(&self, other: &Const) -> Const
    {
        assert!(!self.is_floating_point());
        assert!(!other.is_floating_point());

        let typ = self.get_type();
        let intrm_self = ConstIntermediateFormat::from(self);
        let other = other.convert_to(&Type::Int);
        let intrm_other = ConstIntermediateFormat::from(&other);

        let mut result = intrm_self.clone();

        match intrm_self.value_type {
            ValueType::SignedInt => { result.i_val = Some(intrm_self.i_val.unwrap() >> intrm_other.i_val.unwrap()); },
            ValueType::UnsignedInt => { result.u_val = Some(intrm_self.u_val.unwrap() >> intrm_other.i_val.unwrap()); },
            ValueType::FloatingPoint => { panic!(">> Cannot be applied to floating point"); }
        }

        result.to_const(&typ)
    }



    pub fn logical_and(&self, other: &Const) -> Const
    {
        if self.is_false() || other.is_false() {
            Const::from_i64(0, &Type::Int)
        }
        else {
            Const::from_i64(1, &Type::Int)
        }
    }


    pub fn logical_or(&self, other: &Const) -> Const
    {
        if self.is_true() || other.is_true() {
            Const::from_i64(1, &Type::Int)
        }
        else {
            Const::from_i64(0, &Type::Int)
        }
    }

    pub fn lt(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::I(IntegerConst::ConstInt(a)), Const::I(IntegerConst::ConstInt(b))) => Const::I(IntegerConst::ConstInt((*a < *b) as i32)),
            (Const::I(IntegerConst::ConstUInt(a)), Const::I(IntegerConst::ConstUInt(b))) => Const::I(IntegerConst::ConstInt((*a < *b) as i32)),
            (Const::I(IntegerConst::ConstLong(a)), Const::I(IntegerConst::ConstLong(b))) => Const::I(IntegerConst::ConstInt((*a < *b) as i32)),
            (Const::I(IntegerConst::ConstULong(a)), Const::I(IntegerConst::ConstULong(b))) => Const::I(IntegerConst::ConstInt((*a < *b) as i32)),
            (Const::F(FloatingPointConst::ConstDouble(a)), Const::F(FloatingPointConst::ConstDouble(b))) => Const::I(IntegerConst::ConstInt((*a < *b) as i32)),

            (_, _) => {
                let common_typ = get_const_common_type(self, other);
                let comm_a = self.convert_to(&common_typ);
                let comm_b = other.convert_to(&common_typ);
                let intrm_a = ConstIntermediateFormat::from(&comm_a);
                let intrm_b = ConstIntermediateFormat::from(&comm_b);
                assert_eq!(intrm_a.value_type, intrm_b.value_type);
                let mut intrm_r = intrm_a.clone();

                intrm_r.value_type = ValueType::SignedInt;
                match intrm_a.value_type {
                    ValueType::SignedInt        => *intrm_r.i_val.as_mut().unwrap() = (intrm_a.i_val.unwrap() < intrm_b.i_val.unwrap()) as i64,
                    ValueType::UnsignedInt      => *intrm_r.i_val.as_mut().unwrap() = (intrm_a.u_val.unwrap() < intrm_b.u_val.unwrap()) as i64,
                    ValueType::FloatingPoint    => *intrm_r.i_val.as_mut().unwrap() = (intrm_a.f_val.unwrap() < intrm_b.f_val.unwrap()) as i64,
                };

                intrm_r.to_const(&Type::Int)
            }
        }
    }


    pub fn gt(&self, other: &Const) -> Const
    {
        other.lt(self)
    }


    pub fn le(&self, other: &Const) -> Const
    {
        other.lt(self).logical_not()
    }

    pub fn ge(&self, other: &Const) -> Const
    {
        self.lt(other).logical_not()
    }

    pub fn eq(&self, other: &Const) -> Const
    {
        self.le(other).logical_and(&other.le(self))
    }

    pub fn ne(&self, other: &Const) -> Const
    {
        self.eq(other).logical_not()
    }


}


#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Plus,
    Complement,
    Negate,
    LogicalNot,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement
}

impl Precedence for UnaryOperator {
    fn precedence(&self) -> u32 {
        60
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    LogicalOr,
    LogicalAnd,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    RemainderAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
    ConditionalMiddle //Not a real binary operator
}

impl Precedence for BinaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            BinaryOperator::Assign |
            BinaryOperator::AddAssign |
            BinaryOperator::SubtractAssign |
            BinaryOperator::MultiplyAssign |
            BinaryOperator::DivideAssign |
            BinaryOperator::RemainderAssign |
            BinaryOperator::BitwiseAndAssign |
            BinaryOperator::BitwiseOrAssign |
            BinaryOperator::BitwiseXorAssign |
            BinaryOperator::ShiftLeftAssign |
            BinaryOperator::ShiftRightAssign => 1,

            BinaryOperator::ConditionalMiddle => 3,

            BinaryOperator::LogicalOr => 5,

            BinaryOperator::LogicalAnd => 10,

            BinaryOperator::BitwiseOr => 15,

            BinaryOperator::BitwiseXor => 20,

            BinaryOperator::BitwiseAnd => 25,

            BinaryOperator::Equal|
            BinaryOperator::NotEqual => 30,

            BinaryOperator::LessThan|
            BinaryOperator::LessOrEqual |
            BinaryOperator::GreaterThan |
            BinaryOperator::GreaterOrEqual => 35,

            BinaryOperator::ShiftLeft|
            BinaryOperator::ShiftRight => 40,

            BinaryOperator::Add|
            BinaryOperator::Subtract => 45,

            BinaryOperator::Multiply |
            BinaryOperator::Divide   |
            BinaryOperator::Remainder => 50
        }
    }
}


pub fn get_noncompound_operator(compond_binary_operator: &BinaryOperator) -> Result<BinaryOperator, String>
{
    let noncompund_binop = match compond_binary_operator {
        BinaryOperator::AddAssign           => BinaryOperator::Add,
        BinaryOperator::SubtractAssign      => BinaryOperator::Subtract,
        BinaryOperator::MultiplyAssign      => BinaryOperator::Multiply,
        BinaryOperator::DivideAssign        => BinaryOperator::Divide,
        BinaryOperator::RemainderAssign     => BinaryOperator::Remainder,
        BinaryOperator::BitwiseAndAssign    => BinaryOperator::BitwiseAnd,
        BinaryOperator::BitwiseOrAssign     => BinaryOperator::BitwiseOr,
        BinaryOperator::BitwiseXorAssign    => BinaryOperator::BitwiseXor,
        BinaryOperator::ShiftLeftAssign     => BinaryOperator::ShiftLeft,
        BinaryOperator::ShiftRightAssign    => BinaryOperator::ShiftRight,
        _ => { return Err(format!("Expected compound assignment operator. got: '{:?}'", compond_binary_operator)); }
    };

    Ok(noncompund_binop)
}

#[derive(Debug, Clone)]
pub enum Label {
    Goto(String),
    Case(TypedExpression), //Expression must be constant
    Default,
    ResolvedCase(String)
}


#[derive(Debug)]
pub enum Statement {
    Stmnt(Option<Vec<Label>> /* labels */, UnlabeledStatement)
}

#[derive(Debug)]
pub enum ForInit {
    InitDecl(VariableDeclaration),
    InitExp(Option<TypedExpression>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum BreakType {
    Loop,
    Switch
}


#[derive(Debug)]
pub enum UnlabeledStatement {
    Return(TypedExpression),
    Goto(String),
    If(TypedExpression, Box<Statement> /* then */, Option<Box<Statement>> /* else */),
    Break(Option<BreakType>, Option<String> /* loop/switch label */),
    Continue(Option<String> /* loop label */),
    While(TypedExpression /* condition */, Box<Statement> /* body */, Option<String> /* loop label */),
    DoWhile(Box<Statement> /* body */, TypedExpression /* condition */, Option<String> /* loop label */),
    For(ForInit, Option<TypedExpression> /* condition */, Option<TypedExpression> /* post */, Box<Statement> /* body */, Option<String> /* loop label */),
    Switch(TypedExpression, Box<Statement> /* body */, Option<String> /* switch label */, HashMap<IntegerConst, String>, /* case constants and global labels */ Option<String> /* default_label */ ),
    Compound(Block),
    Expr(TypedExpression),
    Null
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Long,
    UInt,
    ULong,
    Double,
    FuncType(Vec<Type> /* param_types */, Box<Type> /* ret_type */, bool /* has_body, i.e. defined */)
}

impl Type
{
    pub fn is_func(&self) -> bool
    {
        match self {
            Type::FuncType(_,_,_) => true,
            _ => false
        }
    }

    pub fn to_string(&self) -> String
    {
        match self {
            Type::Int               => String::from("int"),
            Type::Long              => String::from("long"),
            Type::UInt              => String::from("uint"),
            Type::ULong             => String::from("ulong"),
            Type::Double            => String::from("double"),

            Type::FuncType(param_types, ret_type,_) => {
                    assert!(!ret_type.is_func());
                    let param_types_str : Vec<String> = param_types.iter().map(|typ| typ.to_string()).collect();
                    format!("{}({})", ret_type.to_string(), param_types_str.join(", "))
            }
        }
    }

    pub fn alignment(&self) -> usize
    {
        match self {
            Type::Int   |
            Type::UInt  => 4,

            Type::Long  |
            Type::ULong |
            Type::Double => 8,

            _ => { panic!("Attempt to call alignment() for function type"); }
        }
    }

   pub fn size(&self) -> usize
    {
        match self {
            Type::Int   |
            Type::UInt  => 4,

            Type::Long  |
            Type::ULong |
            Type::Double => 8,

            _ => { panic!("Attempt to call size() for function type"); }
        }
    }

    pub fn is_signed_integer(&self) -> bool
    {
        match self {
            Type::Int   |
            Type::Long => true,

            Type::UInt  |
            Type::ULong => false,

            Type::Double => false,

            _ => { panic!("Attempt to call is_signed_integer() for function type"); }
        }
    }

    pub fn is_unsigned_integer(&self) -> bool
    {
        match self {
            Type::Int   |
            Type::Long => false,

            Type::UInt  |
            Type::ULong => true,

            Type::Double => false,

            _ => { panic!("Attempt to call is_unsigned_integer() for function type"); }
        }
    }

    pub fn is_integer(&self) -> bool
    {
        match self {
            Type::Int   |
            Type::Long  |
            Type::UInt  |
            Type::ULong
                => true,

            Type::Double
                => false,

            _
                => { panic!("Attempt to call is_integer() for '{:?}'", self); }
        }
    }

    pub fn is_floating_point(&self) -> bool
    {
        match self {
            Type::Int   |
            Type::Long  |
            Type::UInt  |
            Type::ULong
                => false,

            Type::Double
                => true,

            _
                => { panic!("Attempt to call is_floating_point() for '{:?}'", self); }
        }
    }

}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageClass {
    Static,
    Extern
}


#[derive(Debug)]
pub enum VariableDeclaration {
    Declarant(
        String /* var_name */,
        Option<TypedExpression> /* initializer */,
        Type /* var_type */,
        Option<StorageClass>
    )
}


#[derive(Debug)]
pub enum FunctionDeclaration {
    Declarant(
        String /* func_name */,
        Vec<String> /* args */,
        Option<Block> /* body */,
        Type /* must Type::FuncType */,
        Option<StorageClass>
    )
}


#[derive(Debug)]
pub enum Declaration {
    VarDecl(VariableDeclaration),
    FunDecl(FunctionDeclaration)
}


#[derive(Debug)]
pub enum BlockItem {
    D(Declaration),
    S(Statement)
}

#[derive(Debug)]
pub enum Block {
    Blk(Vec<BlockItem>)
}



#[derive(Debug)]
pub enum Program {
    ProgramDefinition(Vec<Declaration>)
}


/*                    GRAMMAR

<program>               ::= [ <function-declaration> ]*
<declaration>           ::= <variable-declaration> | <function-declaration>
<variable-declaration>  ::= <specifier>+ <identifier> [ "=" <exp> ] ";"
<function-declaration>  ::= <specifier>+ <identifier> "(" [<param-list>] )" ( <block> | ";" )
<specifier>             ::= <type-specifier> | "static" | "extern"
<type-specifier>        ::= "int" | "long" | "unsigned" | "signed" | "double"
<param-list>            ::= "" |
                            "void" |
                            <type-specifier> <identifier> ["," <type-specifier> <identifier>"]*
<block>                 ::= "{" [<block_item> *] "}"
<block_item>            ::= <statement>|<declaration>
<label>                 ::= <id> ":" | "case" <exp> ":" | "default" ":"
<statement>             ::= [<label> *] <unlbld_statement>
<unlbld_statement>      ::= ";" |
                            <exp> ";" |
                            "return" <exp> ";" |
                            "goto" <id> ";"   |
                            "break" ";" |
                            "continue" ";"  |
                            "if" "(" <exp> ")" <statement> ["else" <statement> ] |
                            "while" "(" <exp> ")" <statement> |
                            "do" <statement> "while" "(" <exp> ")" |
                            "for" "(" for_init ";" [<exp>] ";" [<exp>] ")" <statement>  |
                            "switch" "(" <exp ")" <statement>
                            <block>
<for_init>              ::= <variable-declaration> | [<exp>]
<exp>                   ::= <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
<factor>                ::= <const> | <identifier> |
                            "(" <type-specifier>+ ")" <factor>
                            <unop> <factor> | "(" <exp> ")" |
                            <inc_dec> <factor> | <factor> <inc_dec> |
                            <identifier> "(" [ <argument-list > ] ")"
<argument-list>         ::= <exp> ["," <exp>]*
<unop>                  ::= "+"  | "-" | "~" | "!"
<inc_dec>               ::= "++" | "--"
<binop>                 ::= "-"  | "+" | "*" | "/" | "%" |
                            "<<" | ">>" | "|"  | "&"  | "^"  |
                            "&&" | "||" |
                            "==" | "!=" | "<"  | "<=" | ">"  | ">=" |
                            "="  | "+=" | "-=" | "*=" | "/=" | "%=" |
                            "|=" | "&=" |
                            "<<="| ">>="
<const>                 ::= <int> | <long> | <uint> | <ulong> | <double>
<identifier>            ::= ? Token::Identifier ?
<int>                   ::= ? Token::IntConstant ?
<long>                  ::= ? Token::LongConstant ?
<uint>                  ::= ? Token::UIntConstant ?
<ulong>                 ::= ? Token::ULongConstant ?
<double>                ::= ? Token::DoubleConstant ?

*/
