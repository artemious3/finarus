use crate::server::ServerError;
use l1::common::auth::Login;
use l1::common::salary::*;
use l1::common::transaction::TransactionEndPoint;
use l1::common::Money;
use serde::{Serialize, Deserialize};
use std::collections::hash_map::*;


#[derive(Default)]
pub struct SalaryService {
    pub salary_requests: HashMap<Login, Vec<SalaryClientRequest>>, // enterprise name -> list of
    // salary requests
    pub salary_projects: HashMap<Login, SalaryProject>, // enterprise name -> one salary project
}

impl SalaryService {
    pub fn salary_request(
        &mut self,
        req: SalaryClientRequest,
    ) -> Result<(), ServerError> {
        match self.salary_requests.entry(req.enterprise_name.clone()) {
            Entry::Vacant(en) => {
                let vec = en.insert(Vec::new());
                vec.push(req);
            }
            Entry::Occupied(mut en) => {
                let vec = en.get_mut();
                vec.push(req)
            }
        }
        Ok(())
    }

    pub fn salary_accept_decline(
        &mut self,
        enterprise_name: Login,
        req: &SalaryAcceptRequest,
    ) -> Result<(), ServerError> {
        let salary_proj =
            self.salary_projects
                .get_mut(&enterprise_name)
                .ok_or(ServerError::BadRequest(
                    "No salary project for this enterprise".to_string(),
                ))?;

        if !salary_proj.accepted{
            return Err(ServerError::Forbidden("Salary project not acccepted".to_string()))
        }

        if let Entry::Occupied(mut en) = self.salary_requests.entry(enterprise_name) {
            let salary_requests = en.get_mut();

            let request = if req.idx < salary_requests.len() {
                salary_requests.swap_remove(req.idx)
            } else {
                return Err(ServerError::BadRequest("Index out of range".to_string()));
            };

            if salary_requests.len() == 0 {
                en.remove();
            }

            if req.accept {
                salary_proj.employees.push(Employee {
                    salary: req.salary,
                    login: request.client_login,
                    account: request.account,
                });
            }

            Ok(())
        } else {
            Err(ServerError::BadRequest("Enterprise not found".to_string()))
        }
    }


    pub fn init_salary_proj(&mut self, enterprise_name: Login, account : TransactionEndPoint ){
        self.salary_projects.insert(enterprise_name, SalaryProject{
            employees : Vec::new(),
            enterprise_accoint : account, 
            accepted : false
        });
    }



}
