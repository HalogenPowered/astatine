// Class file version number
pub const JVM_CLASS_FILE_MAJOR_VERSION: u16 = 61;
pub const JVM_CLASS_FILE_MINOR_VERSION: u16 = 0;

// All class file versions
pub const JAVA_VERSION_1_1: u16 = 45;
pub const JAVA_VERSION_1_2: u16 = 46;
pub const JAVA_VERSION_1_3: u16 = 47;
pub const JAVA_VERSION_1_4: u16 = 48;
pub const JAVA_VERSION_1_5: u16 = 49;
pub const JAVA_VERSION_6: u16 = 50;
pub const JAVA_VERSION_7: u16 = 51;
pub const JAVA_VERSION_8: u16 = 52;
pub const JAVA_VERSION_9: u16 = 53;
pub const JAVA_VERSION_10: u16 = 54;
pub const JAVA_VERSION_11: u16 = 55;
pub const JAVA_VERSION_12: u16 = 56;
pub const JAVA_VERSION_13: u16 = 57;
pub const JAVA_VERSION_14: u16 = 58;
pub const JAVA_VERSION_15: u16 = 59;
pub const JAVA_VERSION_16: u16 = 60;
pub const JAVA_VERSION_17: u16 = 61;

// Access flags
pub const JVM_ACC_PUBLIC: u32 = 0x0001;
pub const JVM_ACC_PRIVATE: u32 = 0x0002;
pub const JVM_ACC_PROTECTED: u32 = 0x0004;
pub const JVM_ACC_STATIC: u32 = 0x0008;
pub const JVM_ACC_FINAL: u32 = 0x0010;
pub const JVM_ACC_SYNCHRONIZED: u32 = 0x0020;
pub const JVM_ACC_SUPER: u32 = 0x0020;
pub const JVM_ACC_VOLATILE: u32 = 0x0040;
pub const JVM_ACC_BRIDGE: u32 = 0x0040;
pub const JVM_ACC_TRANSIENT: u32 = 0x0080;
pub const JVM_ACC_VARARGS: u32 = 0x0080;
pub const JVM_ACC_NATIVE: u32 = 0x0100;
pub const JVM_ACC_INTERFACE: u32 = 0x0200;
pub const JVM_ACC_ABSTRACT: u32 = 0x0400;
pub const JVM_ACC_STRICT: u32 = 0x0800;
pub const JVM_ACC_SYNTHETIC: u32 = 0x1000;
pub const JVM_ACC_ANNOTATION: u32 = 0x2000;
pub const JVM_ACC_ENUM: u32 = 0x4000;
pub const JVM_ACC_MODULE: u32 = 0x8000;

// Basic types

// On the spec
pub const JVM_T_BOOLEAN: u8 = 4;
pub const JVM_T_CHAR: u8 = 5;
pub const JVM_T_FLOAT: u8 = 6;
pub const JVM_T_DOUBLE: u8 = 7;
pub const JVM_T_BYTE: u8 = 8;
pub const JVM_T_SHORT: u8 = 9;
pub const JVM_T_INT: u8 = 10;
pub const JVM_T_LONG: u8 = 11;

// Internals
pub const JVM_T_OBJECT: u8 = 12;
pub const JVM_T_ARRAY: u8 = 13;
pub const JVM_T_VOID: u8 = 14;
pub const JVM_T_ADDRESS: u8 = 15;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum BasicType {
    Boolean = JVM_T_BOOLEAN,
    Char = JVM_T_CHAR,
    Float = JVM_T_FLOAT,
    Double = JVM_T_DOUBLE,
    Byte = JVM_T_BYTE,
    Short = JVM_T_SHORT,
    Int = JVM_T_INT,
    Long = JVM_T_LONG,
    Object = JVM_T_OBJECT,
    Array = JVM_T_ARRAY,
    Void = JVM_T_VOID,
    Address = JVM_T_ADDRESS
}

impl BasicType {
    pub fn from(value: u8) -> Option<BasicType> {
        if value < JVM_T_BOOLEAN || value > JVM_T_LONG {
            return None;
        }
        Some(BasicType::from_unchecked(value))
    }

    pub fn from_unchecked(value: u8) -> BasicType {
        // SAFETY: Caller must guarantee that JVM_T_BOOLEAN <= value <= JVM_T_ADDRESS.
        unsafe { std::mem::transmute(value) }
    }
}

// Constant pool entry tags

pub const JVM_CONSTANT_UTF8: u16 = 1;
pub const JVM_CONSTANT_INTEGER: u16 = 3;
pub const JVM_CONSTANT_FLOAT: u16 = 4;
pub const JVM_CONSTANT_LONG: u16 = 5;
pub const JVM_CONSTANT_DOUBLE: u16 = 6;
pub const JVM_CONSTANT_CLASS: u16 = 7;
pub const JVM_CONSTANT_STRING: u16 = 8;
pub const JVM_CONSTANT_FIELD_REF: u16 = 9;
pub const JVM_CONSTANT_METHOD_REF: u16 = 10;
pub const JVM_CONSTANT_INTERFACE_METHOD_REF: u16 = 11;
pub const JVM_CONSTANT_NAME_AND_TYPE: u16 = 12;
pub const JVM_CONSTANT_METHOD_HANDLE: u16 = 15;
pub const JVM_CONSTANT_METHOD_TYPE: u16 = 16;
pub const JVM_CONSTANT_DYNAMIC: u16 = 17;
pub const JVM_CONSTANT_INVOKE_DYNAMIC: u16 = 18;
pub const JVM_CONSTANT_MODULE: u16 = 19;
pub const JVM_CONSTANT_PACKAGE: u16 = 20;

// MethodHandle reference kinds

pub const JVM_REF_GET_FIELD: u8 = 1;
pub const JVM_REF_GET_STATIC: u8 = 2;
pub const JVM_REF_PUT_FIELD: u8 = 3;
pub const JVM_REF_PUT_STATIC: u8 = 4;
pub const JVM_REF_INVOKE_VIRTUAL: u8 = 5;
pub const JVM_REF_INVOKE_STATIC: u8 = 6;
pub const JVM_REF_INVOKE_SPECIAL: u8 = 7;
pub const JVM_REF_NEW_INVOKE_SPECIAL: u8 = 8;
pub const JVM_REF_INVOKE_INTERFACE: u8 = 9;

// StackMapTable type item numbers

pub const JVM_ITEM_TOP: u8 = 0;
pub const JVM_ITEM_INTEGER: u8 = 1;
pub const JVM_ITEM_FLOAT: u8 = 2;
pub const JVM_ITEM_DOUBLE: u8 = 3;
pub const JVM_ITEM_LONG: u8 = 4;
pub const JVM_ITEM_NULL: u8 = 5;
pub const JVM_ITEM_UNINITIALIZED_THIS: u8 = 6;
pub const JVM_ITEM_OBJECT: u8 = 7;
pub const JVM_ITEM_UNINITIALIZED: u8 = 8;

// Type signatures

pub const JVM_SIGNATURE_SEPARATOR: char = '/';
pub const JVM_SIGNATURE_ARRAY: char = '[';
pub const JVM_SIGNATURE_BYTE: char = 'B';
pub const JVM_SIGNATURE_CHAR: char = 'C';
pub const JVM_SIGNATURE_DOUBLE: char = 'D';
pub const JVM_SIGNATURE_FLOAT: char = 'F';
pub const JVM_SIGNATURE_INT: char = 'I';
pub const JVM_SIGNATURE_LONG: char = 'J';
pub const JVM_SIGNATURE_CLASS: char = 'L';
pub const JVM_SIGNATURE_SHORT: char = 'S';
pub const JVM_SIGNATURE_VOID: char = 'V';
pub const JVM_SIGNATURE_BOOLEAN: char = 'Z';
pub const JVM_SIGNATURE_END_CLASS: char = ';';
pub const JVM_SIGNATURE_METHOD: char = '(';
pub const JVM_SIGNATURE_END_METHOD: char = ')';

// Special method names

pub const JVM_CLASS_INITIALIZER_NAME: &str = "<clinit>";
pub const JVM_OBJECT_INITIALIZER_NAME: &str = "<init>";

// Class file attribute names

pub const JVM_ATTRIBUTE_CONSTANT_VALUE: &str = "ConstantValue";
pub const JVM_ATTRIBUTE_CODE: &str = "Code";
pub const JVM_ATTRIBUTE_STACK_MAP_TABLE: &str = "StackMapTable";
pub const JVM_ATTRIBUTE_EXCEPTIONS: &str = "Exceptions";
pub const JVM_ATTRIBUTE_INNER_CLASSES: &str = "InnerClasses";
pub const JVM_ATTRIBUTE_ENCLOSING_METHOD: &str = "EnclosingMethod";
pub const JVM_ATTRIBUTE_SYNTHETIC: &str = "Synthetic";
pub const JVM_ATTRIBUTE_SIGNATURE: &str = "Signature";
pub const JVM_ATTRIBUTE_SOURCE_FILE: &str = "SourceFile";
pub const JVM_ATTRIBUTE_SOURCE_DEBUG_EXTENSION: &str = "SourceDebugExtension";
pub const JVM_ATTRIBUTE_LINE_NUMBER_TABLE: &str = "LineNumberTable";
pub const JVM_ATTRIBUTE_LOCAL_VARIABLE_TABLE: &str = "LocalVariableTable";
pub const JVM_ATTRIBUTE_LOCAL_VARIABLE_TYPE_TABLE: &str = "LocalVariableTypeTable";
pub const JVM_ATTRIBUTE_DEPRECATED: &str = "Deprecated";
pub const JVM_ATTRIBUTE_RUNTIME_VISIBLE_ANNOTATIONS: &str = "RuntimeVisibleAnnotations";
pub const JVM_ATTRIBUTE_RUNTIME_INVISIBLE_ANNOTATIONS: &str = "RuntimeInvisibleAnnotations";
pub const JVM_ATTRIBUTE_RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS: &str = "RuntimeVisibleParameterAnnotations";
pub const JVM_ATTRIBUTE_RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS: &str = "RuntimeInvisibleParameterAnnotations";
pub const JVM_ATTRIBUTE_RUNTIME_VISIBLE_TYPE_ANNOTATIONS: &str = "RuntimeVisibleTypeAnnotations";
pub const JVM_ATTRIBUTE_RUNTIME_INVISIBLE_TYPE_ANNOTATIONS: &str = "RuntimeInvisibleTypeAnnotations";
pub const JVM_ATTRIBUTE_ANNOTATION_DEFAULT: &str = "AnnotationDefault";
pub const JVM_ATTRIBUTE_BOOTSTRAP_METHODS: &str = "BootstrapMethods";
pub const JVM_ATTRIBUTE_METHOD_PARAMETERS: &str = "MethodParameters";
pub const JVM_ATTRIBUTE_MODULE: &str = "Module";
pub const JVM_ATTRIBUTE_MODULE_PACKAGES: &str = "ModulePackages";
pub const JVM_ATTRIBUTE_MODULE_MAIN_CLASS: &str = "ModuleMainClass";
pub const JVM_ATTRIBUTE_NEST_HOST: &str = "NestHost";
pub const JVM_ATTRIBUTE_NEST_MEMBERS: &str = "NestMembers";
pub const JVM_ATTRIBUTE_RECORD: &str = "Record";
pub const JVM_ATTRIBUTE_PERMITTED_SUBCLASSES: &str = "PermittedSubclasses";

// Opcodes (bytecode)

pub const JVM_OPCODE_NOP: u8 = 0;
pub const JVM_OPCODE_ACONST_NULL: u8 = 1;
pub const JVM_OPCODE_ICONST_M1: u8 = 2;
pub const JVM_OPCODE_ICONST_0: u8 = 3;
pub const JVM_OPCODE_ICONST_1: u8 = 4;
pub const JVM_OPCODE_ICONST_2: u8 = 5;
pub const JVM_OPCODE_ICONST_3: u8 = 6;
pub const JVM_OPCODE_ICONST_4: u8 = 7;
pub const JVM_OPCODE_ICONST_5: u8 = 8;
pub const JVM_OPCODE_LCONST_0: u8 = 9;
pub const JVM_OPCODE_LCONST_1: u8 = 10;
pub const JVM_OPCODE_FCONST_0: u8 = 11;
pub const JVM_OPCODE_FCONST_1: u8 = 12;
pub const JVM_OPCODE_FCONST_2: u8 = 13;
pub const JVM_OPCODE_DCONST_0: u8 = 14;
pub const JVM_OPCODE_DCONST_1: u8 = 15;
pub const JVM_OPCODE_BIPUSH: u8 = 16;
pub const JVM_OPCODE_SIPUSH: u8 = 17;
pub const JVM_OPCODE_LDC: u8 = 18;
pub const JVM_OPCODE_LDC_W: u8 = 19;
pub const JVM_OPCODE_LDC2_W: u8 = 20;
pub const JVM_OPCODE_ILOAD: u8 = 21;
pub const JVM_OPCODE_LLOAD: u8 = 22;
pub const JVM_OPCODE_FLOAD: u8 = 23;
pub const JVM_OPCODE_DLOAD: u8 = 24;
pub const JVM_OPCODE_ALOAD: u8 = 25;
pub const JVM_OPCODE_ILOAD_0: u8 = 26;
pub const JVM_OPCODE_ILOAD_1: u8 = 27;
pub const JVM_OPCODE_ILOAD_2: u8 = 28;
pub const JVM_OPCODE_ILOAD_3: u8 = 29;
pub const JVM_OPCODE_LLOAD_0: u8 = 30;
pub const JVM_OPCODE_LLOAD_1: u8 = 31;
pub const JVM_OPCODE_LLOAD_2: u8 = 32;
pub const JVM_OPCODE_LLOAD_3: u8 = 33;
pub const JVM_OPCODE_FLOAD_0: u8 = 34;
pub const JVM_OPCODE_FLOAD_1: u8 = 35;
pub const JVM_OPCODE_FLOAD_2: u8 = 36;
pub const JVM_OPCODE_FLOAD_3: u8 = 37;
pub const JVM_OPCODE_DLOAD_0: u8 = 38;
pub const JVM_OPCODE_DLOAD_1: u8 = 39;
pub const JVM_OPCODE_DLOAD_2: u8 = 40;
pub const JVM_OPCODE_DLOAD_3: u8 = 41;
pub const JVM_OPCODE_ALOAD_0: u8 = 42;
pub const JVM_OPCODE_ALOAD_1: u8 = 43;
pub const JVM_OPCODE_ALOAD_2: u8 = 44;
pub const JVM_OPCODE_ALOAD_3: u8 = 45;
pub const JVM_OPCODE_IALOAD: u8 = 46;
pub const JVM_OPCODE_LALOAD: u8 = 47;
pub const JVM_OPCODE_FALOAD: u8 = 48;
pub const JVM_OPCODE_DALOAD: u8 = 49;
pub const JVM_OPCODE_AALOAD: u8 = 50;
pub const JVM_OPCODE_BALOAD: u8 = 51;
pub const JVM_OPCODE_CALOAD: u8 = 52;
pub const JVM_OPCODE_SALOAD: u8 = 53;
pub const JVM_OPCODE_ISTORE: u8 = 54;
pub const JVM_OPCODE_LSTORE: u8 = 55;
pub const JVM_OPCODE_FSTORE: u8 = 56;
pub const JVM_OPCODE_DSTORE: u8 = 57;
pub const JVM_OPCODE_ASTORE: u8 = 58;
pub const JVM_OPCODE_ISTORE_0: u8 = 59;
pub const JVM_OPCODE_ISTORE_1: u8 = 60;
pub const JVM_OPCODE_ISTORE_2: u8 = 61;
pub const JVM_OPCODE_ISTORE_3: u8 = 62;
pub const JVM_OPCODE_LSTORE_0: u8 = 63;
pub const JVM_OPCODE_LSTORE_1: u8 = 64;
pub const JVM_OPCODE_LSTORE_2: u8 = 65;
pub const JVM_OPCODE_LSTORE_3: u8 = 66;
pub const JVM_OPCODE_FSTORE_0: u8 = 67;
pub const JVM_OPCODE_FSTORE_1: u8 = 68;
pub const JVM_OPCODE_FSTORE_2: u8 = 69;
pub const JVM_OPCODE_FSTORE_3: u8 = 70;
pub const JVM_OPCODE_DSTORE_0: u8 = 71;
pub const JVM_OPCODE_DSTORE_1: u8 = 72;
pub const JVM_OPCODE_DSTORE_2: u8 = 73;
pub const JVM_OPCODE_DSTORE_3: u8 = 74;
pub const JVM_OPCODE_ASTORE_0: u8 = 75;
pub const JVM_OPCODE_ASTORE_1: u8 = 76;
pub const JVM_OPCODE_ASTORE_2: u8 = 77;
pub const JVM_OPCODE_ASTORE_3: u8 = 78;
pub const JVM_OPCODE_IASTORE: u8 = 79;
pub const JVM_OPCODE_LASTORE: u8 = 80;
pub const JVM_OPCODE_FASTORE: u8 = 81;
pub const JVM_OPCODE_DASTORE: u8 = 82;
pub const JVM_OPCODE_AASTORE: u8 = 83;
pub const JVM_OPCODE_BASTORE: u8 = 84;
pub const JVM_OPCODE_CASTORE: u8 = 85;
pub const JVM_OPCODE_SASTORE: u8 = 86;
pub const JVM_OPCODE_POP: u8 = 87;
pub const JVM_OPCODE_POP2: u8 = 88;
pub const JVM_OPCODE_DUP: u8 = 89;
pub const JVM_OPCODE_DUP_X1: u8 = 90;
pub const JVM_OPCODE_DUP_X2: u8 = 91;
pub const JVM_OPCODE_DUP2: u8 = 92;
pub const JVM_OPCODE_DUP2_X1: u8 = 93;
pub const JVM_OPCODE_DUP2_X2: u8 = 94;
pub const JVM_OPCODE_SWAP: u8 = 95;
pub const JVM_OPCODE_IADD: u8 = 96;
pub const JVM_OPCODE_LADD: u8 = 97;
pub const JVM_OPCODE_FADD: u8 = 98;
pub const JVM_OPCODE_DADD: u8 = 99;
pub const JVM_OPCODE_ISUB: u8 = 100;
pub const JVM_OPCODE_LSUB: u8 = 101;
pub const JVM_OPCODE_FSUB: u8 = 102;
pub const JVM_OPCODE_DSUB: u8 = 103;
pub const JVM_OPCODE_IMUL: u8 = 104;
pub const JVM_OPCODE_LMUL: u8 = 105;
pub const JVM_OPCODE_FMUL: u8 = 106;
pub const JVM_OPCODE_DMUL: u8 = 107;
pub const JVM_OPCODE_IDIV: u8 = 108;
pub const JVM_OPCODE_LDIV: u8 = 109;
pub const JVM_OPCODE_FDIV: u8 = 110;
pub const JVM_OPCODE_DDIV: u8 = 111;
pub const JVM_OPCODE_IREM: u8 = 112;
pub const JVM_OPCODE_LREM: u8 = 113;
pub const JVM_OPCODE_FREM: u8 = 114;
pub const JVM_OPCODE_DREM: u8 = 115;
pub const JVM_OPCODE_INEG: u8 = 116;
pub const JVM_OPCODE_LNEG: u8 = 117;
pub const JVM_OPCODE_FNEG: u8 = 118;
pub const JVM_OPCODE_DNEG: u8 = 119;
pub const JVM_OPCODE_ISHL: u8 = 120;
pub const JVM_OPCODE_LSHL: u8 = 121;
pub const JVM_OPCODE_ISHR: u8 = 122;
pub const JVM_OPCODE_LSHR: u8 = 123;
pub const JVM_OPCODE_IUSHR: u8 = 124;
pub const JVM_OPCODE_LUSHR: u8 = 125;
pub const JVM_OPCODE_IAND: u8 = 126;
pub const JVM_OPCODE_LAND: u8 = 127;
pub const JVM_OPCODE_IOR: u8 = 128;
pub const JVM_OPCODE_LOR: u8 = 129;
pub const JVM_OPCODE_IXOR: u8 = 130;
pub const JVM_OPCODE_LXOR: u8 = 131;
pub const JVM_OPCODE_IINC: u8 = 132;
pub const JVM_OPCODE_I2L: u8 = 133;
pub const JVM_OPCODE_I2F: u8 = 134;
pub const JVM_OPCODE_I2D: u8 = 135;
pub const JVM_OPCODE_L2I: u8 = 136;
pub const JVM_OPCODE_L2F: u8 = 137;
pub const JVM_OPCODE_L2D: u8 = 138;
pub const JVM_OPCODE_F2I: u8 = 139;
pub const JVM_OPCODE_F2L: u8 = 140;
pub const JVM_OPCODE_F2D: u8 = 141;
pub const JVM_OPCODE_D2I: u8 = 142;
pub const JVM_OPCODE_D2L: u8 = 143;
pub const JVM_OPCODE_D2F: u8 = 144;
pub const JVM_OPCODE_I2B: u8 = 145;
pub const JVM_OPCODE_I2C: u8 = 146;
pub const JVM_OPCODE_I2S: u8 = 147;
pub const JVM_OPCODE_LCMP: u8 = 148;
pub const JVM_OPCODE_FCMPL: u8 = 149;
pub const JVM_OPCODE_FCMPG: u8 = 150;
pub const JVM_OPCODE_DCMPL: u8 = 151;
pub const JVM_OPCODE_DCMPG: u8 = 152;
pub const JVM_OPCODE_IFEQ: u8 = 153;
pub const JVM_OPCODE_IFNE: u8 = 154;
pub const JVM_OPCODE_IFLT: u8 = 155;
pub const JVM_OPCODE_IFGE: u8 = 156;
pub const JVM_OPCODE_IFGT: u8 = 157;
pub const JVM_OPCODE_IFLE: u8 = 158;
pub const JVM_OPCODE_IF_ICMPEQ: u8 = 159;
pub const JVM_OPCODE_IF_ICMPNE: u8 = 160;
pub const JVM_OPCODE_IF_ICMPLT: u8 = 161;
pub const JVM_OPCODE_IF_ICMPGE: u8 = 162;
pub const JVM_OPCODE_IF_ICMPGT: u8 = 163;
pub const JVM_OPCODE_IF_ICMPLE: u8 = 164;
pub const JVM_OPCODE_IF_ACMPEQ: u8 = 165;
pub const JVM_OPCODE_IF_ACMPNE: u8 = 166;
pub const JVM_OPCODE_GOTO: u8 = 167;
pub const JVM_OPCODE_JSR: u8 = 168;
pub const JVM_OPCODE_RET: u8 = 169;
pub const JVM_OPCODE_TABLESWITCH: u8 = 170;
pub const JVM_OPCODE_LOOKUPSWITCH: u8 = 171;
pub const JVM_OPCODE_IRETURN: u8 = 172;
pub const JVM_OPCODE_LRETURN: u8 = 173;
pub const JVM_OPCODE_FRETURN: u8 = 174;
pub const JVM_OPCODE_DRETURN: u8 = 175;
pub const JVM_OPCODE_ARETURN: u8 = 176;
pub const JVM_OPCODE_RETURN: u8 = 177;
pub const JVM_OPCODE_GETSTATIC: u8 = 178;
pub const JVM_OPCODE_PUTSTATIC: u8 = 179;
pub const JVM_OPCODE_GETFIELD: u8 = 180;
pub const JVM_OPCODE_PUTFIELD: u8 = 181;
pub const JVM_OPCODE_INVOKEVIRTUAL: u8 = 182;
pub const JVM_OPCODE_INVOKESPECIAL: u8 = 183;
pub const JVM_OPCODE_INVOKESTATIC: u8 = 184;
pub const JVM_OPCODE_INVOKEINTERFACE: u8 = 185;
pub const JVM_OPCODE_INVOKEDYNAMIC: u8 = 186;
pub const JVM_OPCODE_NEW: u8 = 187;
pub const JVM_OPCODE_NEWARRAY: u8 = 188;
pub const JVM_OPCODE_ANEWARRAY: u8 = 189;
pub const JVM_OPCODE_ARRAYLENGTH: u8 = 190;
pub const JVM_OPCODE_ATHROW: u8 = 191;
pub const JVM_OPCODE_CHECKCAST: u8 = 192;
pub const JVM_OPCODE_INSTANCEOF: u8 = 193;
pub const JVM_OPCODE_MONITORENTER: u8 = 194;
pub const JVM_OPCODE_MONITOREXIT: u8 = 195;
pub const JVM_OPCODE_WIDE: u8 = 196;
pub const JVM_OPCODE_MULTIANEWARRAY: u8 = 197;
pub const JVM_OPCODE_IFNULL: u8 = 198;
pub const JVM_OPCODE_IFNONNULL: u8 = 199;
pub const JVM_OPCODE_GOTO_W: u8 = 200;
pub const JVM_OPCODE_JSR_W: u8 = 201;
