function toggle_visibility(id) {
  var e = document.getElementById(id);
  if(e.classList.contains('hide'))
     e.classList.remove('hide');
  else
     e.classList.add('hide');
}