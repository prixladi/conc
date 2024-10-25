# Basic usage example

This example requires having the application installed and the **daemon** running.

- Open a terminal in the current directory
- Run `concc upsert` - this will create a new project from the `./conc.json` file definition
- Run `concc ps basic-example` to get the status of the new project
- Run `concc start basic-example dir` to start the dir service in the basic-example project
- Run `concc start basic-example` to start all services in the basic-example project
- Run `concc logs basic-example` to `tail` logs of all the services in the basic-example project
- Run `concc logs basic-example dir -r` to show the raw path to the logfile of dir service that you can open in editor/viewer of your choice.
- Run `concg` to open the **gui**, it should be relatively intuitive after **cli** introduction.
