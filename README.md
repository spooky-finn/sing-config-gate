### Overview

This project provides:

* a **web server** that serves client configuration files
* a **Telegram bot** that generates and distributes a **unique access link for each user**

The current implementation focuses on **VLESS + Reality** setups, but the architecture can be generalized to support other protocols or configurations if needed.

### Motivation

The project uses **sing-box** instead of **xray**.

The main reason is that **sing-box provides more flexible routing configuration**. This is particularly useful in environments with heavy network filtering (e.g., Russian internet censorship).

With sing-box it is possible to:

* route **domestic traffic directly**
* route **international traffic through a proxy**

Routing can be configured using:

* full domain names or **regular expressions**
* **domain suffixes** (e.g., `.com`, `.org`, `.ru`)

This allows selective proxying and improves performance for local services while still bypassing external restrictions.
