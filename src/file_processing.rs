use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use blockstack_lib::{
    chainstate::stacks::StacksTransaction, 
    core::mempool::MemPoolTxInfo
};
use rusqlite::Result;

use crate::error_handler::MyError;

pub struct SmartContract {
    pub name_of_smart_contract: MemPoolTxInfo,
    pub tx_info_tx: StacksTransaction,
    pub filename: String,
}

impl SmartContract {
    pub fn contract_name(&self) -> String {
        let contract_str = format!("{:?}", self.name_of_smart_contract);
        let parts: Vec<&str> = contract_str.split(',').collect();

        let mut contract_name = "";
        for part in parts {
            if part.contains("ContractName") {
                let start = part.find('"').unwrap() + 1;
                let end = part.rfind('"').unwrap();
                contract_name = &part[start..end];
                break;
            }
        }

        contract_name.to_string()
    }

    // fn create_file(&self) -> Result<(), MyError> {
    //     let file = File::create(&self.filename)?;
    //     serde_json::to_writer_pretty(file, &self.tx_info_tx)?;
    //     Ok(())
    // }

    fn append_data(&self) -> Result<(), MyError> {
        let file = OpenOptions::new().append(true).open(&self.filename)?;
        serde_json::to_writer_pretty(file, &self.tx_info_tx)?;
        Ok(())
    }
}

pub fn separate_files(transactions: &[StacksTransaction], smart_contract: SmartContract) {
    let mut named_transactions: Vec<(String, Vec<StacksTransaction>)> = vec![];

    for tx in transactions.iter() {
        let name = smart_contract.contract_name();

        #[allow(unused_assignments)]
        let mut name_2 = String::new();

        if name.is_empty() {
            name_2 = "add/NO_NAME.json".to_string()
        } else {
            name_2 = format!("add/{}.json", name);
        }

        println!("\t\t{name_2}");

        if let Some((_, ref mut txs)) = named_transactions.iter_mut().find(|(n, _)| n == &name_2) {
            txs.push(tx.clone());
        } else {
            named_transactions.push((name_2.clone(), vec![tx.clone()]));
        }
    }

    for (name, txs) in named_transactions.iter() {
        let path = Path::new(&name);
        if path.exists() {
            let mut temporary_vector: Vec<StacksTransaction> = {
                File::open(path)
                    .map_err(Into::<MyError>::into)
                    .and_then(|file| serde_json::from_reader(file).map_err(Into::into))
                    .unwrap_or_else(|err| {
                        log::warn!(
                            "failed to load transactions: {err}\nFile '{name}' will be recreated"
                        );
                        vec![]
                    })
            };
            temporary_vector.extend_from_slice(txs.as_slice());
            File::create(path)
                .and_then(|mut f| {
                    f.write_all(
                        serde_json::to_string_pretty(&temporary_vector)
                            .unwrap()
                            .as_bytes(),
                    )
                })
                .unwrap();
        } else {
            File::create(path).unwrap();
            for tx in txs.iter() {
                let sc = SmartContract {
                    name_of_smart_contract: smart_contract.name_of_smart_contract.clone(),
                    tx_info_tx: tx.clone(),
                    filename: name.clone(),
                };
                sc.append_data().unwrap();
            }
        }
    }
}
