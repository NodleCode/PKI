# W3F Grant Milestones

| Number | Deliverable | Files | Notes |
| ------------- | ------------- | ------------- |------------- |
| 1. | Development of the TCR and Root Of Trust pallets as well as a CLI to interact with the Root Of Trust. | The pallets are in the `./pallets` folder while the cli is in `./cli` | Instead of adding functions to manage adminsitrators adding authorities we decided to make the Root Of Trust compatible with the `membership` module traits and have the TCR support the `ChangeMembers` trait to allow for an easier integration with other pallets. | 
| 2.  | A POC IoT firmware / application to demo the use of the Root Of Trust for an IoT use case. | TBD | TBD | 
