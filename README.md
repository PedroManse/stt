# STT

###### This describes what the language *should* be, *not necessarily* what it is

Pronounced as "stah", Stt is a simple scripting language made to be embedded in other rust programs.

It's also made to be as simple as possible to parse and execute.

It's recommended usage is to import a standard libraby and make your own
definitions for easier interoperability with the container software system.
Then, before every run modify the stack to include that 'input' variables, and
after executing the user's script, getting the stack values as output.

It's also possible to add rust bindings to be executable by user script

