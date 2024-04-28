This program automatically polls the currently connected network's SSID and updates the hosts file with matching entries (if any exist). If there are no matches, it clears the previous entry added to the hosts file (if that was the case). Upon execution of the program, a default config file (which is `config.toml`) will be created. The configuration is quite simple. Update the `hosts_path` value with the hosts file path if it is different in your system. Then add an SSID name under the `[ssid]` section. This program also adds itself to startup so it will be automatically started on boot.

**Do NOT forget to run the program as administrator; otherwise, it won't be able to update the hosts file.**

Default config:
```toml
hosts_path = 'C:\Windows\System32\drivers\etc\hosts'


[ssid]
example = """
    # Redirect requests to example.com to 192.168.1.1.
    192.168.1.1 example.com


    # Redirect requests to sub.example.com to 192.168.1.2.
    192.168.1.2 sub.example.com
"""
```
In the above default config, if you are connected to an SSID with the name `example`, its value will be added to the hosts file.

It is also possible to run the program with the following parameters which, will print the output to `logs.txt`:
* `ssid`: Get the SSID of the currently connected network. 
* `version`: Get the version of the program.

---
**Currently, only Windows is supported.**