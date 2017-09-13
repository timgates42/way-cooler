//! Utility methods and constructors for Lua objects

use std::fmt::Display;
use std::convert::From;
use rlua::{self, Lua, Table, MetaMethod, UserData, Value, UserDataMethods,
           FromLua};
use super::signal::{connect_signal, emit_signal};

/// All Lua objects can be cast to this.
pub struct Object<'lua> {
    table: Table<'lua>
}


// TODO Move this to TryFrom and check that data exists and is
// the right type.
// Can't just yet cause TryFrom is still nightly...

impl <'lua> From<Table<'lua>> for Object<'lua> {
    fn from(table: Table<'lua>) -> Self {
        Object { table }
    }
}

impl <'lua> Object<'lua> {
    pub fn signals(&self) -> rlua::Table {
        self.table.get::<_, Table>("signals")
            .expect("Object table did not have signals defined!")
    }
}


pub fn add_meta_methods<T: UserData + Display>(methods: &mut UserDataMethods<T>) {
    methods.add_meta_method(MetaMethod::ToString, |lua, obj: &T, _: ()| {
        Ok(lua.create_string(&format!("{}", obj)))
    });

    // TODO Add {connect,disconnect,emit}_signal methods
}

pub fn default_index<'lua>(lua: &'lua Lua, (obj_table, index): (Table<'lua>, Value<'lua>))
                       -> rlua::Result<Value<'lua>> {
    // Look up in metatable first
    if let Some(meta) = obj_table.get_metatable() {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {},
                val => return Ok(val)
            }
        }
    }
    // TODO Handle non string indexing?
    // double check C code
    let index = String::from_lua(index, lua)?;
    // TODO FIXME handle special "valid" property
    match index.as_str() {
        "connect_signal" => {
            let func = lua.create_function(
                |lua, (obj_table, signal, func): (Table, String, rlua::Function)| {
                    connect_signal(lua, obj_table.into(), signal, func)
            });
            func.bind(obj_table).map(rlua::Value::Function)
        },
        "emit_signal" => {
            let func = lua.create_function(
                |lua, (obj_table, signal, args): (Table, String, rlua::Value)| {
                    // TODO FIXME this seems wrong to always pass the object table in,
                    // but maybe that's always how object signal emitting should work?
                    // Look this up, double check!
                    emit_signal(lua, &obj_table.clone().into(), signal, obj_table)
                });
            func.bind(obj_table).map(rlua::Value::Function)
        }
        index => {
            let err_msg = format!("Could not find index \"{:#?}\"", index);
            warn!("{}", err_msg);
            Err(rlua::Error::RuntimeError(err_msg))
        }
    }
}
