﻿use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::values::Value;
use crate::vm::vm::ControlFlow;

#[derive(Debug, Clone)]
pub struct Frame {
    pub(crate) map: BTreeMap<String, Value>,
    pub root: Option<Arc<Mutex<Frame>>>,
    pub closure: Option<Arc<Mutex<Frame>>>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            map: BTreeMap::new(),
            root: Option::None,
            closure: Option::None
        }
    }

    pub fn has(&self, name: String) -> bool {
        if self.map.contains_key(&name) {
            true
        } else {
            let mut current = self.root.clone();
            while let Some(ref current_ref) = current.clone() {
                let guard = current_ref.lock().unwrap();
                if guard.has(name.clone()) {
                    return true;
                } else {
                    current = guard.root.clone();
                }
            }
            false
        }
    }

    pub fn lookup(&self, address: Address, name: String) -> Result<Value, ControlFlow> {
        // checking current frame
        if let Some(val) = self.map.get(&name) {
            return Ok(val.clone())
        }
        if let Some(ref closure_ref) = self.closure {
            let guard = closure_ref.lock().unwrap();
            if guard.has(name.clone()) {
                return guard.lookup(address, name);
            }
        }
        // checking others
        let mut current = self.root.clone();
        while let Some(ref current_ref) = current.clone() {
            let guard = current_ref.lock().unwrap();
            if guard.has(name.clone()) {
                return guard.lookup(address.clone(), name.clone())
            }
            current = guard.root.clone();
        }
        // error
        Err(ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        )))
    }

    pub fn set(&mut self, address: Address, name: String, val: Value) -> Result<(), ControlFlow> {
        // checking current frame
        if self.map.contains_key(&name) {
            self.map.insert(name, val.clone());
            return Ok(());
        }
        if let Some(ref closure_ref) = self.closure {
            let mut guard = closure_ref.lock().unwrap();
            if guard.has(name.clone()) {
                guard.set(address.clone(), name.clone(), val.clone())?;
                return Ok(());
            }
        }
        // checking others
        let mut current = self.root.clone();
        while let Some(ref current_ref) = current.clone(){
            let mut guard = current_ref.lock().unwrap();
            if guard.has(name.clone()) {
                guard.set(address.clone(), name.clone(), val.clone())?;
                return Ok(());
            }
            current = guard.root.clone();
        }
        // error
        Err(ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        )))
    }

    pub fn define(&mut self, address: Address, name: String, val: Value) -> Result<(), ControlFlow> {
        // checking current frame
        if self.map.contains_key(&name) {
            self.map.insert(name.clone(), val);
            Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address,
                format!("already defined: {:?}", name),
                "check variable overrides.".to_string()
            )))
        } else {
            self.map.insert(name, val);
            Ok(())
        }
    }

    pub fn set_root(&mut self, frame: Arc<Mutex<Frame>>) {
        // current roo
        if self.root.is_none() {
            self.root = Some(frame.clone());
            return;
        }
        // other roots
        let mut last_root = self.root.clone();
        while last_root.is_some() {
            let root_cloned = last_root.clone().unwrap();
            let guard = root_cloned.lock().unwrap();
            let new_root = guard.root.clone();
            if new_root.is_some() {
                last_root = new_root;
            } else {
                break;
            }
        }
        last_root.unwrap().lock().unwrap().root = Option::Some(frame.clone());
    }
}