# Server for hosting the static website

This is a simple server for hosting the static website. It is written in Rust with Rocket framework.
On load, it downloads the website from the S3 bucket and serves it to the client.
There is also a several endpoints for uploading files from tar.gz of zip archive.

To run the server, you need to set the following environment variables:
> AWS_ACCESS_KEY_ID=<access_key>;
>
> AWS_ENDPOINT_URL_S3=<endpoint_url>;
>
> AWS_REGION=<region>;
>
> AWS_SECRET_ACCESS_KEY=<secret_key>
>
> DOKI__AUTH__PASSWORD=<password>;
>
> DOKI__AUTH__USERNAME=<username>;
>
> DOKI__FS__TEMP_DIR=./tmp; // Directory for storing the uploaded files, it's also can be set in config.yaml

## Endpoints

Every endpoint requires basic authentication. The username and password are set in the environment variables.

Header: `Authorization: Basic <base64(username:password)>`

**POST /api/admin/update** - Update the website from the S3 bucket.

```shell
curl --location --request POST 'http://localhost:8000/api/admin/update' \
--header 'Authorization: Basic dXNlcjpwYXNz'
```

**POST /api/admin/upload** - Upload the file from the archive. The archive should be tar.gz or zip.

send tar.gz archive:

```shell
curl --location 'http://localhost:8000/api/admin/upload' \
--header 'Content-Type: application/gzip' \
--header 'Authorization: Basic dXNlcjpwYXNz' \
--data '/path/to/site.tar.gz'
```

send zip archive:

```shell
curl --location 'http://localhost:8000/api/admin/upload' \
--header 'Content-Type: application/zip' \
--header 'Authorization: Basic dXNlcjpwYXNz' \
--data '/path/to/site.zip'
```