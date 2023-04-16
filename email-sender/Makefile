build-EmailSender:
	cd $(PWD); cargo lambda build --release
	cp $(PWD)/target/lambda/email-sender/bootstrap $(ARTIFACTS_DIR)