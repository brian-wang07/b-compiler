use rustc_hash::FxHashMap;

//interner based symbol table. Each identifier (variable, parameter, function, label, extern declarations) 
//are hashed for O(1) lookup and to avoid internal string copying. 
//They are also stored in a vec for proper error reporting. Since B does not support
//block scopes (variables defined in a function are in scope for the entire function),
//a seperate ScopeId is not necessary; instead, keeping track of global and function scopes
//sufficies.
//ex: a = 5. check if a exists in function env -> check if a exists in global env -> report error.
//FxHashMap is used for its faster, non-cryptographic hashing function.

pub enum SymbolError {
  UndefinedSymbol,
  DefinitionError,
}
pub enum SymbolKind { Auto, Extrn, Param, Function, Label }

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct SymbolId(u32); //unique variables have unique internal id's

pub enum Location {
  //location enum to define where variables are stored in VM. 
  Local {slot: u32}, //function parameters decay into local parameters when function is in scope
  Global {index: u32}, //globals should be stored in different address than locals
  Function {func_index: u32}, //functions are stored in a seperate lookup table; location indexes to position in table
  //TODO: Not like the B language, but maybe implement namespacing? Would need to store Import as (module, name), add name mangling and namespace op.
  Import {import_index: u32}, //Extrn declarations. Same as functions, stored in import table
}

pub struct Interner {
  strings: Vec<String>,
  lookup: FxHashMap<String, SymbolId>,
}


pub struct Symbol {   
  //holds metadata for identifiers
  name: SymbolId,
  kind: SymbolKind,
  size: u32, //1 for variables
  slot: Location,
}

pub struct FunctionEnv {
  //map identifier internal id -> identifier metadata
  locals: FxHashMap<SymbolId, Symbol>,
  params: Vec<SymbolId>,
}

pub struct GlobalEnv {
  //top level declarations (global extrn, functions)
  symbols: FxHashMap<SymbolId, Symbol>,
}

pub struct SymbolTable {
  interner: Interner,
  global: GlobalEnv,
}

impl Interner {

  pub fn new() -> Self {
    Self {
      strings: Vec::new(),
      lookup: FxHashMap::default()
    }
  }
  ///Check if variable has been declared already within scope. If it has, return the the Id of the variable.
  /// Else, Create a new unique id, and insert it into the lookup table.
  pub fn intern(&mut self, name: &str) -> SymbolId {
    match self.lookup.get(name) {
      Some(&v) => v,
      None => {
        //length of strings is the number of unique variables defined. use len to get the next unique id
        let id = SymbolId(self.strings.len() as u32);
        //clone name (&str type) to String 
        let owned = name.to_owned();
        self.strings.push(owned.clone());
        self.lookup.insert(owned, id);
        id
      }
    }
  }

  ///get variable name from symbolId. Used for Error handling, as compiler internally never uses variable names
  pub fn resolve(&self, id: SymbolId) -> &str {
    //never panics as symbolId is always a valid index
    &self.strings[id.0 as usize]
  }

  ///query lookup without inserting
  pub fn id_of(&self, name: &str) -> Option<SymbolId> {
    self.lookup.get(name)
    .map(|x| *x) //deref Some(&SymbolId); cheap clone
  }

  //helpers:
  fn len(&self) -> usize {
    self.strings.len()
  }

  fn is_empty(&self) -> bool {
    self.strings.len() == 0
  }
}


impl SymbolTable {
  pub fn new() -> Self {
    Self
  }
}




