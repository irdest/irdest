# Setup Guides

This section of the manual focuses on practical setup instructions for
an Irdest network with Ratman and other applications.  Each guide is
tailored to a specific use-case, with some being more broad than
others.

Ideally your use-case will be covered.  In case it is not, hopefully
you can adapt one of the existing guides to your settings.  But please
also feel free to [talk to us](https://irde.st/community/) about what
the documentation should include!

| Bitfield              | Description      |
|-----------------------|------------------|
| `0000 0000 0000 0000` | Announcement     |
| `0000 0000 0001 xxxx` | **Reserved**     |
| `0000 0000 0010 0001` | SyncScopeRequest |
| `0000 0000 001x xxx0` | **Reserved**     |
| `0000 0000 0100 0001` | SourceRequest    |
| `0000 0000 0010 0010` | SourceResponse   |
| `0000 0000 01xx xx11` | **Reserved**     |
| `0000 0000 1000 0001` | PushNotice       |
| `0000 0000 1000 0010` | DenyNotice       |
| `0000 0000 1000 0011` | PullRequest      |
| `0000 0000 1000 0100` | PullResponse     |
| `0xxx xxxx 1xxx x1x1` | **Reserved**     |
| `1000 0000 0000 0001` | LinkLeapNotice   |
    
