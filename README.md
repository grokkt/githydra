### Don't use, project incomplete

The idea for this project was a tool to help users manage multiple github accounts from one machine, since I personally found that workflow to be overly complex

After working on it for a while I realized this abstraction is kind of pointless, and a user would be better off and have an easier time just reading a tutorial on how to manage it themselves

Was fun to work on nonetheless, got some good experience with clap.rs as well as file system management in Rust

If you think this project would infact be useful for you, feel free to create an issue and I can continue to work on it

# How to manage multiple github accounts from one machine manually:

### Create a new SSH key for your github account
- Example with ssh-keygen:
```
ssh-keygen -t rsa -b 4096 -C "associated email or comment"
```
- This will generate 2 files, find the one ending in `.pub` (important!) and copy that over to your github account in account settings

### Create an ssh config file (`~/.ssh/config`)
- Add an entry for the new github account:
```
Host some_ssh_alias
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_rsa_mykey
```
- The `IdentifyFile` should be the path to the private key (the one that doesn't end in .pub)
- The `some_ssh_alias` can be anything, would make sense to make it your github profile name

### Set the correct git remote for a project you want to be associated with the new github account
```
git remote add origin git@some_ssh_alias:username/repo.git
```
- `some_ssh_alias` should map to the `Host` in your ssh config file
- `username/repo.git` is the path for your repo on github

### (Optional) Update the git config email and name on your repo
- `git config user.email myemail@example.com`
- `git config user.name my_name`

