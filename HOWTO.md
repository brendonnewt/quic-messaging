# How to Set Up and Connect to the Local Database

## Starting the Database

Use Docker Compose to start the database:

```bash
docker compose -f [local.docker-compose.yml](http://_vscodecontentref_/1) up -d
```

- To stop the database, use: `docker compose -f docker/local.docker-compose.yml down`
- To clear the data from the database, use: `docker compose -f docker/local.docker-compose.yml down -v`

## Connecting to Your Local Database in Rust Rover

1. Open the "Database" tab (if you don't see it, go to View > Tool Windows > Database)
2. Click the "+" button and add a Data Source > MySQL
3. Enter "root" for the username
4. Enter "password" for the password
5. If there is a warning at the bottom of the dialog to "Download missing driver files", click the "Download" button to do so.
6. Click "Test Connection" to verify the connection succeeded. If it did, click "OK" to close the dialog.

## Setting Up the Tables

1. Locate the `table_definitions.txt` file in the `server/utils` directory.

2. Open the MySQL console by clicking on the data source in the **Database** tab.

3. Copy the queries from the `table_definitions.txt` file.

4. Paste the queries into the MySQL console and execute them to set up the required tables.

## ENV File

To configure the application, create an `.env` file in the root of the `server` directory. The file should include the following environment variables:

```env
DATABASE_URL=mysql://root:password@localhost:3307/messaging
SECRET=anythingyouwant
