# Installing

Install AWS CMD and AWS SAM
```
sam build --guided
sam deploy
```

Then perform the following (once off) manual actions:
1. Go to "AWS Certificate manager console", and add required CNAME records to your DNS server
2. Go to "AWS SES console" -> "Verified Identies"  and add CNAME records to verify domain
3. Add MX record to your DNS server to send all emails to AWS (TODO link instructions)
4. Go to AWS SES console and set the `sinln-email-input-receipt` to active:

