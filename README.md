# Chameleon 

Chameleon provides better content discovery by using wappalyzer's set of technology fingerprints alongside custom wordlists tailored to each detected technologies.

The tool is highly customizable and allows users to add in their own custom wordlists, extensions or fingerprints.

The full documentation is available on:
https://youst.in/posts/context-aware-conent-discovery-with-chameleon/

## Installation

### Linux 64-bit and MacOS
```
curl -sL https://raw.githubusercontent.com/iustin24/chameleon/master/install.sh | bash
```
Running the script will create the directory `~/.config/chameleon/` and download the config file and custom wordlists.


## Example Usage:

### Tech Scan + Directory Bruteforce:
```
> chameleon --url https://example.com -a
```
<br>
<p align="center">
  <img width="1200" src="_img/Screenshot%202022-09-10%20at%2000.57.25.png">
</p>
<br>

### Options

```
OPTIONS:
    -a, --tech-detect
            Automatically detect technologies with wappalyzer and adapt wordlist

    -A, --auto-calibrate
            Automatically calibrate filtering options (default: false)

    -c, --mc <MATCHCODE>...
            Match HTTP status codes from response - Comma separated list [default:
            200,204,301,302,307,401,403,405]

    -C, --fc <FILTERCODE>...
            Filter HTTP status codes from response - Comma separated list

    -h, --help
            Print help information

    -i, --include tech <TECHS>
            Technology to be included, even if its not detected by wappalyzer. ( -i PHP,IIS )

    -J, --json
            Save the output as json

    -k, --config <CONFIG>
            Config file to use [default: ~/.config/chameleon/config.toml]

    -L, --hosts-file <HOSTS_FILE>
            List of hosts to scan

    -o, --output <OUTPUT>
            Save the output into a file

    -s, --ms <MATCHSIZE>...
            Match HTTP response size. Comma separated list of sizes

    -S, --fs <FILTERSIZE>...
            Filter HTTP response size. Comma separated list of sizes

    -t, --concurrency <CONCURRENCY>
            Number of concurrent threads ( default: 200 ) [default: 40]

    -T, --tech url <TECH_URL>
            URL which will be scanned for technologies. By default, this is the same as '-u',
            however it can be changed using '-T'

    -u, --url <URL>
            url to scan

    -U, --user-agent <USERAGENT>
            Change the value for the user-agent header [default: "Chameleon /
            https://github.com/iustin24/chameleon"]

    -V, --version
            Print version information

    -w, --wordlist <WORDLIST>
            Main wordlist to use for bruteforcing

    -W, --small-wordlist <SMALL_WORDLIST>
            Wordlist used to generate files by adding extensions ( FUZZ.%ext )

```

## Config file

Chameleon uses the config file located in `~/.config/chameleon/config.yaml`. 

### Changing the default wordlists:

If no wordlist is provided, chameleon will use the wordlist specified in `main_wordlist` from the config file. ( Default: ~/.config/chameleon/wordlists/raft-medium-words.txt )

When detecting technologies with characteristic extensions, chameleon will generate a wordlist by like so ( FUZZ.%ext ). Chameleon will use the wordlist specified in `small_wordlist` from the config file. ( Default: ~/.config/chameleon/wordlists/raft-medium-words.txt )

### Changing technology wordlists

Example config.yaml with technology specific wordlists:

```
# Technology Specific Wordlists:

Flask="~/.config/chameleon/wordlists/Flask.txt"
Java="~/.config/chameleon/wordlists/Java.txt"
Go="~/.config/chameleon/wordlists/GO.txt"
...
```

### Adding new technology wordlists

Chameleon uses fingerprints from https://github.com/iustin24/wappalyzer/blob/master/apps.json. 
You can add new technology wordlists by taking the name of a technology from `apps.json` and adding it to the config file like so:

<p align="center">
  <img width="600" src="_img/bitrix.png">
</p>

```
# Technology Specific Wordlists:

1C-Bitrix="~/.config/chameleon/wordlists/new_tech_wordlist.txt"
...
```

### Adding new extension fingerprints.

Chameleon generates wordlists using characteristic extensions matching the detected technology. You can add / modify the extensions in the config file like so:

```
# Technology specific Extensions

Microsoft_ASP_NET_ext="aspx,ashx,asmx,asp"
Java_ext="jsp"
CFML_ext="cfm"
Python_ext="py"
PHP_ext="php"
```

## To-do

~~Update the wappalyzer crate to also support the "implies" feature for better technology detection.~~

~~Add auto calibration for filtering~~

Add option to add custom headers.

## Credits 

epi052 - https://github.com/epi052/feroxfuzz/
