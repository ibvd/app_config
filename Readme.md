
I am a big fan of Hashicorp's consul tempaltes.  They come in handy in a lot of use cases, and consul itself is just nice to have around.  But in some cases standing up the consul stack is overkill.  Enter app_config.  

app_config utilizes the AWS AppConfig service and allows you to generate config files and restart services whenever data in AWS AppConfig changes. 

For example, say we wanted to be able to dynamically add wireguard VPN endpoints to a running server or container.  We would need a list of endpoints and their public keys.  If we put that list into AWS AppConfig, then we could update that list and have all servers dynamically update their local config files.


Execution is like so:

```sh
app_config check -f myconfig.toml
```
For now the binary is not wrapped up as a service, so just stick it in cron to periodically check for updates. 


and with myconfig.toml being something like:

```toml
[providers.aws]
application = "myApp"
environment = "dev"
configuration = "wireguard"
client_id = "23"
state_file = "myApp.db"

[hooks.template]
file = "./wg.tmpl"
source_type = "yaml"
out_file = "/etc/wireguard/wg0.conf"

[hooks.command]
command = "wg addconf wg0 <(wg-quick strip /etc/wireguard/wg0)"
```


The template file (i.e. wg.tmpl) in the example above is a [Handlebars](https://handlebarsjs.com/) template.

This is not ready for release, so for examples of use check the tests directory.

New features such as the ability to poll or update based on Azure AppConfig or AWS Parameter store and secret manager are planned next. 



