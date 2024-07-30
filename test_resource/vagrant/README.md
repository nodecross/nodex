## Test for proxy server in windows environment.

### How to use

```bash
vagrant up
```

#### Server side

```bash
vagrant ssh docker
docker compose -f test_resource/compose.yaml --profile e2e up -d
```

#### Agent side

```bash
vagrant rdp agent
```

Connect screen via rdp protocol.

Open PowerShell terminal and Execute a following command.

```ps1
Import-Certificate -Filepath 'C:\\Users\vagrant\nodex\test_resource\fixtures\root-CA.p7b' -CertStoreLocation 'Cert:\CurrentUser\Root'
```

```bash
vagrant ssh agent
powershell
.\nodex\test_resource\vagrant\start_agent.ps1
```
