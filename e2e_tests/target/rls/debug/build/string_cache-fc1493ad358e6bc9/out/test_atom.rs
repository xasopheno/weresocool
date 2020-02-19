pub type TestAtom = :: string_cache :: Atom < TestAtomStaticSet > ;
pub struct TestAtomStaticSet ;
impl :: string_cache :: StaticAtomSet for TestAtomStaticSet { fn get ( ) -> & 'static :: string_cache :: PhfStrSet { static SET : :: string_cache :: PhfStrSet = :: string_cache :: PhfStrSet { key : 732231254413039614u64 , disps : & [ ( 0u32 , 0u32 ) , ( 1u32 , 5u32 ) , ( 0u32 , 5u32 ) ] , atoms : &[
"id",
"br",
"b",
"area",
"head",
"a",
"",
"address",
"body",
"font-weight",
"html" ] , hashes : & [ 1523015218u32 , 3993733920u32 , 3558784543u32 , 1690684349u32 , 388901393u32 , 1359926067u32 , 276304134u32 , 1890225098u32 , 2232698746u32 , 2871158724u32 , 3276045433u32 ] } ;
& SET } fn empty_string_index ( ) -> u32 { 6u32 } } # [ macro_export ] macro_rules ! test_atom {
( "id" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x2u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "br" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x100000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "b" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x200000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "area" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x300000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "head" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x400000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "a" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x500000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x600000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "address" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x700000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "body" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x800000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "font-weight" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0x900000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
( "html" ) => { $ crate :: atom :: tests :: TestAtom { unsafe_data : 0xA00000002u64 , phantom : :: std :: marker :: PhantomData , } } ;
}