# db_new_record_to_json

Directory "/add" (included in ".gitignore") contains: "last_accept_time", "mempool.sqlite", "output.json"

Last accept time example "add/last_accept_time":
```bash
1687841601
```
If the file is empty, missing, or cannot be read, it will be overwritten, variable last_accept_time will be 0, and the program will reindex the database.
