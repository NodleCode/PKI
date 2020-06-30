# W3F Grant Milestones

| Number | Deliverable | Files | Notes |
| ------------- | ------------- | ------------- |------------- |
| 1. | Development of the TCR and Root Of Trust pallets as well as a CLI to interact with the Root Of Trust. | The pallets are in the `./pallets` folder while the cli is in `./cli` | Instead of adding functions to manage adminsitrators adding authorities we decided to make the Root Of Trust compatible with the `membership` module traits and have the TCR support the `ChangeMembers` trait to allow for an easier integration with other pallets. | 
| 2.  | A POC IoT firmware / application to demo the use of the Root Of Trust for an IoT use case. | The code supporting this is fully in NodeJS. We have moved the CLI and all associated code to `./nodejs` which is a simple lerna + yarn monorepo. | We implemented the demo application as a simple web application so that it can run everywhere. The "firmware" is a NodeJS code that can run on a Raspberry Pi, for testing it can also be ran locally however our instructions will make you deploy it on a Raspberry Pi to conform with our initial spec. | 
