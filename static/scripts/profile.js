$('.formInput input, .formInput select').each(function() {
	if ($(this).val() !== '') {
		$('#'+this.id+' + label').animate({
			'fontSize': '0.8rem',
			'top': '-0.7rem',
			'padding': '0.25rem'
		}, 80);
	}
	$(this).focusin(() => {
		$('#'+this.id+' + label').animate({
			'fontSize': '0.8rem',
			'top': '-0.7rem',
			'padding': '0.25rem'
		}, 80);
	});
	$(this).focusout(function() {
		if ($(this).val() === '') {
			$('#'+this.id+' + label').animate({
				'fontSize': '1rem',
				'top': '1rem',
				'padding': 0
			}, 80);
		}
	});
});

$('.dialogBG').click(() => {
	$('.dialog').each(function () {
		$(this).hide();
	});
	$('.dialogBG').hide();
});

$('.dialog').each(function () {
	$(`#${this.id} .footer button.textButton`).click(() => {
		$(this).hide();
		$('.dialogBG').hide();
	});
});

$('#profileBody li:nth-child(2)').click(() => {
	$('.dialogBG').show();
	$('#changeClassDialog').show('fast');
});

$('#profileBody li:nth-child(3)').click(() => {
	$('.dialogBG').show();
	$('#changePasswordDialog').show('fast');
});

$('#profileBody li:nth-child(4)').click(() => {
	$('.dialogBG').show();
	$('#changeEmailDialog').show('fast');
});

$('#profileBody li:nth-child(5)').click(() => {
	$('.dialogBG').show();
	$('#changePrivateDialog').show('fast');
});

function updateProfile(newClass, oldPassword, newPassword, email, profilePic, privateStatus) {
	$.ajax({
		type: 'POST',
		url: '/updateProfile',
		data: {
			class: newClass,
			old_password: oldPassword,
			new_password: newPassword,
			email: email,
			profile_pic: profilePic,
			private: privateStatus,
		},
		success: (result) => {
			console.log(result);
			if (result == 'Success') {
				showToast('Profile updated successfully');
				window.location.href = '/profile';
			} else if (result === 'Error: Wrong old password') {
				$('#changePasswordDialog button:last-child').prop('disabled', false);
				showToast(result, 10000);
			} else {
				showToast(result, 10000);
			}
		}
	});
}

function validURL(str) {
	var pattern = new RegExp('^(https?:\\/\\/)?'+ // protocol
		'((([a-z\\d]([a-z\\d-]*[a-z\\d])*)\\.)+[a-z]{2,}|'+ // domain name
		'((\\d{1,3}\\.){3}\\d{1,3}))'+ // OR ip (v4) address
		'(\\:\\d+)?(\\/[-a-z\\d%_.~+]*)*'+ // port and path
		'(\\?[;&a-z\\d%_.~+=-]*)?'+ // query string
		'(\\#[-a-z\\d_]*)?$','i'); // fragment locator
	return !!pattern.test(str)
}

$('#changeClassDialog button:last-child').click(() => {
	if ($('#class').val() == userDetails['class']) {
		showToast("Please select a different class from the one you're already in", 5000);
	} else {
		showToast('Please wait...');
		$('#changeClassDialog button:last-child').prop('disabled', true);
		updateProfile($('#class').val(), '', '', userDetails['email'], userDetails['profilePic'], userDetails['private']);
	}
});

$('#changePasswordDialog button:last-child').click(() => {
	if ($('#oldPassword').val() === '') {
		showToast("Please enter your old password", 5000);
	} else if ($('#newPassword').val().length <= 4) {
		showToast('The password needs to be longer than 4 characters', 5000);
	} else {
		showToast('Please wait...');
		$('#changePasswordDialog button:last-child').prop('disabled', true);
		updateProfile($('#class').val(), $('#oldPassword').val(), $('#newPassword').val(), userDetails['email'], userDetails['profilePic'], userDetails['private']);
	}
});

$('#changeEmailDialog button:last-child').click(() => {
	if ($('#email').val() === userDetails['email']) {
		showToast("Please give a different email from the one you've already given", 5000);
	} else {
		showToast('Please wait...');
		$('#changeEmailDialog button:last-child').prop('disabled', true);
		updateProfile(userDetails['class'], '', '', $('#email').val(), userDetails['profilePic'], userDetails['private']);
	}
});

$('#changePrivateDialog button:last-child').click(() => {
	if ($('#private').prop('checked') === userDetails['private']) {
		showToast("Please select a different privacy status than the one you're already using", 5000);
	} else {
		showToast('Please wait...');
		$('#changePrivateDialog button:last-child').prop('disabled', true);
		updateProfile(userDetails['class'], '', '', userDetails['email'], userDetails['profilePic'], $('#private').prop('checked'));
	}
});

$('#profilePicture').click(() => {
	var newProfilePic = prompt("Enter the URL for a new profile pic");
	if (validURL(newProfilePic)) {
		showToast('Please wait...');
		updateProfile(userDetails['class'], '', '', userDetails['email'], newProfilePic, userDetails['private']);
	} else {
		showToast('Invalid URL', 5000);
	}
});

$('#signoutButton').click(() => {
	showToast('Please wait...');
	$.ajax({
		type: 'POST',
		url: '/signout',
		success: (result) => {
			console.log(result);
			if (result == 'Success') {
				window.location.href = '/';
			}
		}
	});
});
