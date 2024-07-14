# Motivation
Have a way to script with org files in rust
## Why
I guess because I had the time to write it
# How to use
You have to write your script into the main.rs file and then run the command
```
--file todo.org --history history.org --context todo
```
This is an example, where I take an org file and remove all done Items und update the ones that have the todo tag LOOP
The org file is then again written without these DONE Items and the DONE items get added to a history file
