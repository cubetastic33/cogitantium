if (userDetails.class === 'NOTA' || userDetails.class === 'X A1') {
	$('#A1').prop('checked', true);
}
if (userDetails.class === 'NOTA' || userDetails.class === 'X A2') {
	$('#A2').prop('checked', true);
}
if (userDetails.class === 'NOTA' || userDetails.class === 'X B') {
	$('#B').prop('checked', true);
}

function updateFilter() {
	$('.review.'+this.id).show();
	if (!this.checked) {
		$('.review.'+this.id).hide();
	}
}

$('.filter input').each(updateFilter);

$('.filter input').each(function () {
	$(this).change(updateFilter);
});

function showRegisteredStatus() {
	var name = $('#about').val();
	var about_class = $('#about_class').val();
	if (name.length > 3) {
		$.ajax({
			type: 'POST',
			url: '/registeredStatus',
			data: {
				name: name,
				class: about_class
			},
			success: (result) => {
				console.log(result);
				if (result === 'Registered') {
					$('#newReview .helper-text').attr('class', 'helper-text success');
					$('#newReview .helper-text').text(`${name} of ${about_class} has signed up`);
				} else if (result === 'Not registered') {
					$('#newReview .helper-text').attr('class', 'helper-text warning');
					$('#newReview .helper-text').text(`${name} of ${about_class} has not signed up yet, but you can write about them`);
				} else if (result === 'Self') {
					$('#newReview .helper-text').attr('class', 'helper-text error');
					$('#newReview .helper-text').text(`${name} of ${about_class} is yourself`);
				} else if (result === 'Signed out') {
					window.location.href = '/';
				} else {
					$('#newReview .helper-text').attr('class', 'helper-text error');
					$('#newReview .helper-text').text('An error occurred');
				}
			}
		});
	} else {
		$('#newReview .helper-text').attr('class', 'helper-text');
		$('#newReview .helper-text').empty();
	}
}

$('#about').keyup(showRegisteredStatus);
$('#about_class').change(showRegisteredStatus);

$('#newReview button').click(() => {
	showToast('Please wait...');
	var name = $('#about').val();
	var about_class = $('#about_class').val();
	$.ajax({
		type: 'POST',
		url: '/registeredStatus',
		data: {
			name: name,
			class: about_class
		},
		success: (result) => {
			console.log(result);
			if (result == 'Registered' || 'Not registered') {
				if (result == "Registered") {
					$('#newReview .helper-text').attr('class', 'helper-text success');
					$('#newReview .helper-text').text(`${name} of ${about_class} has signed up`);
				} else {
					$('#newReview .helper-text').attr('class', 'helper-text warning');
					$('#newReview .helper-text').text(`${name} of ${about_class} has not signed up yet, but you can write about them`);
				}
				$.ajax({
					type: 'POST',
					url: '/postReview',
					data: {
						about: $('#about').val(),
						about_class: $('#about_class').val(),
						content: $('#content').val(),
						anonymous: $('#anonymous').prop('checked')
					},
					success: (result) => {
						console.log(result);
						if (result == "Success") {
							window.location.href = '/reviews';
						}
					}
				});
			} else if (result === 'Self') {
				$('#newReview .helper-text').attr('class', 'helper-text error');
				$('#newReview .helper-text').text(`${name} of ${about_class} is yourself`);
			} else if (result === 'Signed out') {
				window.location.href = '/';
			} else {
				$('#newReview .helper-text').attr('class', 'helper-text error');
				$('#newReview .helper-text').text('An error occurred');
			}
		}
	});
});
