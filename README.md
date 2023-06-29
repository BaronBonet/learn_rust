# News parser

https://api.gdeltproject.org/api/v2/doc/doc?query=sourcecountry:FR%20AND%20(%22climate%20change%22%20OR%20%22global%20warming%22)&mode=artlist&maxrecords=250&startdatetime=20230617164918&enddatetime=20230618164918&sort=datedesc&format=json

```
curl -X GET localhost:3000/is-valid-category/climate%20change
```
```
curl -X GET "localhost:3000/get-articles-by-category?categories=climate%20change,environment&inclusive_start_date=2022-01-01&inclusive_end_date=2024-01-01"
```