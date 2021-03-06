use std::error::Error as StdError;
use std::result::Result as StdResult;
use std::fmt::{Display, Formatter, Result as FmtResult};
use serde::ser::{self, Serialize};

#[deprecated(since="0.1.4", note="please use `to_string_pretty` with `PrettyConfig::default()` instead")]
pub mod pretty;
mod value;

/// Serializes `value` and returns it as string.
///
/// This function does not generate any newlines or nice formatting;
/// if you want that, you can use `pretty::to_string` instead.
pub fn to_string<T>(value: &T) -> Result<String>
    where T: Serialize
{
    let mut s = Serializer {
        output: String::new(),
        pretty: (PrettyConfig::basic(false), Pretty { indent: 0 }),
    };
    value.serialize(&mut s)?;
    Ok(s.output)
}

/// Serializes `value` in the recommended RON layout in a pretty way.
pub fn to_string_pretty<T>(value: &T, config: PrettyConfig) -> Result<String>
    where T: Serialize
{
    let mut s = Serializer {
        output: String::new(),
        pretty: (config, Pretty { indent: 0 }),
    };
    value.serialize(&mut s)?;
    Ok(s.output)
}

/// Serialization result.
pub type Result<T> = StdResult<T, Error>;

/// Serialization error.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// A custom error emitted by a serialized value.
    Message(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Error::Message(ref e) => write!(f, "Custom message: {}", e),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref e) => e,
        }
    }
}

/// Pretty serializer state
struct Pretty {
    indent: usize,
}

/// Pretty serializer configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrettyConfig {
    /// New line string
    pub new_line: String,
    /// Indentation string
    pub indentor: String,
    /// Separate tuple members with indentation
    pub separate_tuple_members: bool,
    /// Add struct names
    pub struct_names: bool,
    /// Add spaces after commas between elements in tuples and maps
    pub add_space: bool,
    #[serde(skip)]
    _dummy: (),
}

impl Default for PrettyConfig {
    fn default() -> Self {
        PrettyConfig {
            #[cfg(not(target_os = "windows"))]
            new_line: "\n".to_string(),
            #[cfg(target_os = "windows")]
            new_line: "\r\n".to_string(),
            indentor: "    ".to_string(),
            separate_tuple_members: false,
            struct_names: true,
            add_space: true,
            _dummy: ()
        }
    }
}

impl PrettyConfig {
    pub fn default_with<F>(f: F) -> Self 
        where F: Fn(&mut Self)
    {
        let mut cfg = PrettyConfig::default();
        f(&mut cfg);
        cfg
    }

    pub fn basic(struct_names: bool) -> PrettyConfig {
        PrettyConfig::default_with(|x|{
            x.new_line = String::from("");
            x.indentor = String::from("");
            x.separate_tuple_members = false;
            x.struct_names = struct_names;
            x.add_space = false;
        })
    }
}

/// The RON serializer.
///
/// You can just use `to_string` for deserializing a value.
/// If you want it pretty-printed, take a look at the `pretty` module.
pub struct Serializer {
    output: String,
    pretty: (PrettyConfig, Pretty),
}

impl Serializer {
    fn separate_tuple_members(&self) -> bool {
        self.pretty.0.separate_tuple_members
    }

    fn struct_names(&self) -> bool {
        self.pretty.0.struct_names
    }
    
    fn new_line(&self) -> String {
        self.pretty.0.new_line.clone()
    }

    fn space(&self) -> String {
        if self.pretty.0.add_space { String::from(" ") } else { String::from("") }
    }


    fn start_indent(&mut self) {
        let (ref config, ref mut pretty) = self.pretty;
        pretty.indent += 1;
        self.output += &config.new_line;
    }

    fn indent(&mut self) {
        let (ref config, ref pretty) = self.pretty;
        self.output.extend((0..pretty.indent).map(|_| config.indentor.as_str()));
    }

    fn end_indent(&mut self) {
        let (ref config, ref mut pretty) = self.pretty;
        pretty.indent -= 1;
        self.output.extend((0..pretty.indent).map(|_| config.indentor.as_str()));
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        // TODO optimize
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output += "'";
        if v == '\\' || v == '\'' {
            self.output.push('\\');
        }
        self.output.push(v);
        self.output += "'";
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += "\"";
        for char in v.chars() {
            if char == '\\' || char == '"' {
                self.output.push('\\');
            }
            self.output.push(char);
        }
        self.output += "\"";
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<()> {
        self.output += "None";

        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.output += "Some(";
        value.serialize(&mut *self)?;
        self.output += ")";

        Ok(())
    }

    fn serialize_unit(self) -> Result<()> {
        self.output += "()";

        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        if self.struct_names() {
            self.output += name;

            Ok(())
        } else {
            self.serialize_unit()
        }
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str
    ) -> Result<()> {
        self.output += variant;

        Ok(())
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        if self.struct_names() {
            self.output += name;
        }

        self.output += "(";
        value.serialize(&mut *self)?;
        self.output += ")";
        Ok(())
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T
    ) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.output += variant;
        self.output += "(";

        value.serialize(&mut *self)?;

        self.output += ")";
        Ok(())
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "[";

        self.start_indent();

        Ok(self)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        self.output += "(";

        if self.separate_tuple_members() {
            self.start_indent();
        }

        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleStruct> {
        if self.struct_names() {
            self.output += name;
        }

        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize
    ) -> Result<Self::SerializeTupleVariant> {
        self.output += variant;
        self.output += "(";

        if self.separate_tuple_members() {
            self.start_indent();
        }

        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += "{";

        self.start_indent();

        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _: usize
    ) -> Result<Self::SerializeStruct> {
        if self.struct_names() {
            self.output += name;
        }
        self.output += "(";

        self.start_indent();

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize
    ) -> Result<Self::SerializeStructVariant> {
        self.output += variant;
        self.output += "(";

        self.start_indent();

        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.indent();
        value.serialize(&mut **self)?;
        self.output += ",";
        self.output += &self.new_line();

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.end_indent();

        self.output += "]";
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        if self.separate_tuple_members() {
            self.indent();
        }
        value.serialize(&mut **self)?;
        self.output += ",";
        
        if self.separate_tuple_members() { 
            self.output += &self.new_line(); 
        } else { 
            self.output += &self.space();
        };
        Ok(())
    }

    fn end(self) -> Result<()> {
        if self.separate_tuple_members() {
            self.end_indent();
        } else {
            let len = self.space().len();
            for _ in 0..len {
                self.output.pop();
            }
        }

        self.output += ")";

        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.indent();

        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.output += ":";
        self.output += &self.space();
        value.serialize(&mut **self)?;
        self.output += ",";
        self.output += &self.new_line();
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.end_indent();

        self.output += "}";
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.indent();

        self.output += key;
        self.output += ":";
        self.output += &self.space();
        value.serialize(&mut **self)?;
        self.output += ",";
        self.output += &self.new_line();
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.end_indent();

        self.output += ")";
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeStruct::end(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct EmptyStruct1;

    #[derive(Serialize)]
    struct EmptyStruct2 {}

    #[derive(Serialize)]
    struct MyStruct { x: f32, y: f32 }

    #[derive(Serialize)]
    enum MyEnum {
        A,
        B(bool),
        C(bool, f32),
        D { a: i32, b: i32 }
    }

    #[test]
    fn test_empty_struct() {
        assert_eq!(to_string(&EmptyStruct1).unwrap(), "()");
        assert_eq!(to_string(&EmptyStruct2 {}).unwrap(), "()");
    }

    #[test]
    fn test_struct() {
        let my_struct = MyStruct { x: 4.0, y: 7.0 };

        assert_eq!(to_string(&my_struct).unwrap(), "(x:4,y:7,)");


        #[derive(Serialize)]
        struct NewType(i32);

        assert_eq!(to_string(&NewType(42)).unwrap(), "(42)");

        #[derive(Serialize)]
        struct TupleStruct(f32, f32);

        assert_eq!(to_string(&TupleStruct(2.0, 5.0)).unwrap(), "(2,5,)");
    }

    #[test]
    fn test_option() {
        assert_eq!(to_string(&Some(1u8)).unwrap(), "Some(1)");
        assert_eq!(to_string(&None::<u8>).unwrap(), "None");
    }

    #[test]
    fn test_enum() {
        assert_eq!(to_string(&MyEnum::A).unwrap(), "A");
        assert_eq!(to_string(&MyEnum::B(true)).unwrap(), "B(true)");
        assert_eq!(to_string(&MyEnum::C(true, 3.5)).unwrap(), "C(true,3.5,)");
        assert_eq!(to_string(&MyEnum::D { a: 2, b: 3 }).unwrap(), "D(a:2,b:3,)");
    }

    #[test]
    fn test_array() {
        let empty: [i32; 0] = [];
        assert_eq!(to_string(&empty).unwrap(), "()");
        let empty_ref: &[i32] = &empty;
        assert_eq!(to_string(&empty_ref).unwrap(), "[]");

        assert_eq!(to_string(&[2, 3, 4i32]).unwrap(), "(2,3,4,)");
        assert_eq!(to_string(&(&[2, 3, 4i32] as &[i32])).unwrap(), "[2,3,4,]");
    }

    #[test]
    fn test_map() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert((true, false), 4);
        map.insert((false, false), 123);

        let s = to_string(&map).unwrap();
        s.starts_with("{");
        s.contains("(true,false,):4");
        s.contains("(false,false,):123");
        s.ends_with("}");
    }

    
    #[test]
    fn test_basic_vs_pretty_basic() {

        let my_struct = MyStruct { x: 4.0, y: 7.0 };
        let pretty = to_string_pretty(&my_struct, PrettyConfig::basic(false)).unwrap();
        let basic = to_string(&my_struct).unwrap();

        assert_eq!(basic, "(x:4,y:7,)");
        assert_eq!(basic, pretty);

        #[derive(Serialize)]
        struct NewType(i32);

        let pretty = to_string_pretty(&NewType(42), PrettyConfig::basic(false)).unwrap();
        let basic = to_string(&NewType(42)).unwrap();
        assert_eq!(basic, "(42)");
        assert_eq!(basic, pretty);

        #[derive(Serialize)]
        struct TupleStruct(f32, f32);

        let tuple = TupleStruct(2.0,5.0);
        let pretty = to_string_pretty(&tuple, PrettyConfig::basic(false)).unwrap();
        let basic = to_string(&tuple).unwrap();

        assert_eq!(basic, "(2,5,)");
        assert_eq!(basic, pretty);
    }


    #[test]
    fn test_string() {
        assert_eq!(to_string(&"Some string").unwrap(), "\"Some string\"");
    }

    #[test]
    fn test_char() {
        assert_eq!(to_string(&'c').unwrap(), "'c'");
    }

    #[test]
    fn test_escape() {
        assert_eq!(to_string(&r#""Quoted""#).unwrap(), r#""\"Quoted\"""#);
    }
}
