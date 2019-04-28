# CSV Guillotine

[![Build Status](https://travis-ci.com/forbesmyester/csv-guillotine.svg?branch=master)](https://travis-ci.com/forbesmyester/csv-guillotine)

Often, banks, stockbrokers and other large institutions will offer a feature to download your account history in a CSV. This is good and to be applauded but they often include an extra metadata header at the top of the file explaining what the file is. It may look like the following:

    Account:,****07493
    £4536.24
    £4536.24

    Transaction type,Description,Paid out,Paid in,Balance
    Bank credit YOUR EMPLOYER,Bank credit YOUR EMPLOYER,,£2016.12,£4536.24
    Direct debit CREDIT PLASTIC,CREDIT PLASTIC,£402.98,,£520.12

For many users this is fine as it can still be loaded into a spreadsheet application.

For my use case, I need to download many of these files, which makes up one large data set and these extra metadata headers are quite an issue because I can no longer use [xsv](https://github.com/BurntSushi/xsv) to parse them.

This library is a form of buffer which removes this metadata header. It does this by looking at the amount of field count each row has, for a given number of rows and removes the lines before the maximum is reached.
