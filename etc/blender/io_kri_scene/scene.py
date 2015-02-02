__author__ = ['Dzmitry Malyshau']
__bpydoc__ = 'Scene module of KRI exporter.'

import mathutils
import math
from io_kri.common	import *
from io_kri.action	import *
from io_kri_mesh.mesh	import *


def cook_mat(mat,log):
	textures = []
	for mt in mat.texture_slots:
		if mt == None: continue
		it = mt.texture
		if it == None: continue
		if it.type != 'IMAGE':
			log.log(2, 'w','Texture "%s": type is not IMAGE' % (it.name))
			continue
		if it.image == None:
			log.log(2, 'w','Texture "%s": image is not assigned' % (it.name))
			continue
		textures.append({
			'name'	: mt.name,
			'path'	: it.image.filepath,
			'filter': (1, (2, 3)[it.use_mipmap])[it.use_interpolation],
			'wrap'	: 0,
			'scale'	: list(mt.scale),
			'offset': list(mt.offset),
		})
	kind = 'phong'
	if mat.use_shadeless:
		kind = 'flat'
	elif mat.use_tangent_shading:
		kind = 'anisotropic'
	diff_params = [mat.diffuse_intensity, float(mat.emit), 0.0, mat.alpha]
	spec_params = [mat.specular_intensity, float(mat.specular_hardness), 0.0, mat.specular_alpha]
	return {
		'name'		: mat.name,
		'shader'	: kind,
		'data'		: {
			'Ambient'		: ('scalar',	(mat.ambient,)),
			'DiffuseColor'	: ('color',		list(mat.diffuse_color)),
			'DiffuseParams'	: ('vector',	diff_params),
			'SpecularColor'	: ('color',		list(mat.specular_color)),
			'SpecularParams': ('vector',	spec_params),
		},
		'textures'	: textures,
	}


def cook_space(matrix):
	pos, rot, sca = matrix.decompose()
	scale = (sca.x + sca.y + sca.z)/3.0
	if sca.x*sca.x+sca.y*sca.y+sca.z*sca.z > 0.01 + sca.x*sca.y+sca.y*sca.z+sca.z*sca.x:
		log.log(1,'w', 'Non-uniform scale: (%.1f,%.1f,%.1f)' % sca.to_tuple(1))
	return {
		'pos'	: list(pos),
		'rot'	: [rot.x, rot.y, rot.z, rot.w],
		'scale'	: scale,
	}

def cook_node(ob, log):
	return {
		'name'		: ob.name,
		'space'		: cook_space(ob.matrix_local),
		'children'	: [],
		'actions'	: [],
	}

def cook_camera(cam, log):
	return {	#todo: ortho
		'name'	: cam.name,
		'angle'	: [cam.angle_x, cam.angle_y],
		'range'	: [cam.clip_start, cam.clip_end],
		'actions' : [],
	}

def cook_lamp(lamp, log):
	attenu = [lamp.linear_attenuation, lamp.quadratic_attenuation]
	sphere = False
	params = []
	kind = lamp.type
	if lamp.type in ('SPOT', 'POINT'):
		sphere = lamp.use_sphere
	if lamp.type == 'SPOT':
		params = [lamp.spot_size, lamp.spot_blend, 0.1]
	elif lamp.type == 'AREA':
		params = [lamp.size, lamp.size_y, 0.1]
	return {
		'name'			: lamp.name,
		'kind'			: kind,
		'parameters'	: params,
		'color'			: list(lamp.color),
		'energy'		: lamp.energy,
		'attenuation'	: attenu,
		'distance'		: lamp.distance,
		'spherical'		: sphere,
		'actions'		: [],
	}

def cook_armature(arm,log):
	root = { 'children': [] }
	bones = { '':root }
	for b in arm.bones:
		par = bones['']
		mx = b.matrix_local
		if b.parent:
			par = bones[b.parent.name]
			mx = b.parent.matrix_local.copy().inverted() * mx
		ob = {
			'name'		: b.name,
			'space'		: cook_space(mx),
			'children'	: [],
		}
		par['children'].append(ob)
		bones[b.name] = ob
	return {
		'name'		: arm.name,
		'dual_quat'	: False,
		'bones'		: root['children'],
		'actions'	: [],
	}


def export_value(elem, ofile, num_format, level):
	import collections
	#print('Exporting:',str(elem))
	new_line = '\n%s' % (level * '\t')
	tip = type(elem)
	if tip is tuple:
		last = elem[len(elem)-1]
		if type(last) is dict:	# object
			assert len(elem) <= 3
			name = elem[0]
			if len(elem) == 3:
				name = elem[1]
				ofile.write(elem[0] + '(')
			ofile.write(name)
			if len(last):
				ofile.write('{')
				for key,val in last.items():
					ofile.write('%s\t%s\t: ' % (new_line, key))
					export_value(val, ofile, num_format, level+1)
					ofile.write(',' )
				ofile.write(new_line + '}')
			if len(elem) == 3:
				ofile.write(')')
		else:
			if type(elem[0]) is str:	# enum element
				ofile.write(elem[0])
			if len(elem) > 1:
				ofile.write('(\t')
				for sub in elem[1:]:
					export_value(sub, ofile, num_format, level+1)
					if not (sub is last):
						ofile.write(',\t')
				ofile.write(')')
		#else:
			#raise Exception( 'Composite element %s is unknown' % (str(elem)))
	elif tip is bool:
		ofile.write(('false', 'true')[elem])
	elif tip is int:
		ofile.write(str(elem))
	elif tip is float:
		ofile.write(num_format % (elem))
	elif tip is str:
		ofile.write('~"%s"' % (elem))
	elif tip is list:
		if len(elem):
			subtip = type(elem[0])
			is_class = subtip in (tuple, dict, list, str)
			ofile.write(('[', '~[')[is_class])
			for i,sub in enumerate(elem):
				assert type(sub) == subtip
				if is_class:
					ofile.write(new_line + '\t')
				export_value(sub, ofile, num_format, level+1)
				if i+1 != len(elem):
					ofile.write((', ', ',')[is_class])
			if is_class:
				ofile.write(new_line)
			ofile.write(']')
		else:
			ofile.write('~[]')
	else:
		ofile.write('0')
		raise Exception('Element %s is unknown' % (str(type(elem))))


def export_doc(document,filepath,num_format):
	ofile = open(filepath+'.rs','w')
	ofile.write('use common::*;\n')
	ofile.write('pub fn load()-> Scene	{')
	export_value(document, ofile, num_format, 1)
	ofile.write('}\n')
	ofile.close()


def export_json(document, filepath, num_format):
	import json
	class KriEncoder(json.JSONEncoder):
		def default(self, obj):
			if isinstance(obj, float):
				return num_format % obj
			return json.JSONEncoder.default(self, obj)
	json.encoder.FLOAT_REPR = lambda o: num_format % (o)
	text = json.dumps(document, indent="\t", allow_nan=False, cls=KriEncoder)
	file = open(filepath+'.json', 'w')
	file.write(text)
	file.close()


def save_scene(filepath, context, export_meshes, export_actions, precision):
	glob		= {}
	materials	= []
	nodes		= []
	cameras		= []
	lights		= []
	entities	= []
	# ready...
	import os
	if not os.path.exists(filepath):
		os.makedirs(filepath)
	log	= Logger(filepath+'.log')
	out_mesh, out_act_node, out_act_arm = None, None, None
	collection_mesh, collection_node_anim = 'all', 'nodes'
	if export_meshes:
		out_mesh	= Writer('%s/%s.k3mesh' % (filepath, collection_mesh))
		out_mesh.begin('*mesh')
	if export_actions:
		out_act_node= Writer('%s/%s.k3act' % (filepath, collection_node_anim))
		out_act_node.begin('*action')
	sc = context.scene
	print('Exporting Scene...')
	log.logu(0, 'Scene %s' % (filepath))
	# -globals
	bDegrees = (sc.unit_settings.system_rotation == 'DEGREES')
	if not bDegrees:
		#it's easier to convert on loading than here
		log.log(1, 'w','Radians are not supported')
	if sc.use_gravity:
		gv = sc.gravity
		log.log(1, 'i', 'gravity: (%.1f,%.1f,%.1f)' % (gv.x, gv.y, gv.z))
		glob['gravity'] = list(sc.gravity)
	else:
		glob['gravity'] = [0,0,0]
	# -materials
	for mat in context.blend_data.materials:
		if log.stop:	break
		materials.append(cook_mat(mat, log))
		#save_actions( mat, 'm','t' )
	# -nodes
	node_tree = {}
	for ob in sc.objects:
		node_tree[ob.name] = n = cook_node(ob,log)
		if ob.parent == None:
			nodes.append(n)
		else:
			node_tree[ob.parent.name]['children'].append(n)
	del node_tree
	# steady...
	for ob in sc.objects:
		if log.stop:	break
		# parse node
		if len(ob.modifiers):
			log.log(1,'w','Unapplied modifiers detected on object %s' % (ob.name))
		current = {}
		if ob.type == 'MESH':
			if out_mesh != None:
				(_,face_num) = save_mesh(out_mesh,ob,log)
			else:
				(_,face_num) = collect_attributes(ob.data, None, ob.vertex_groups, True, log)
			offset = 0
			for fn,m in zip(face_num, ob.data.materials):
				if not fn: break
				s = (m.name	if m else '')
				log.logu(1, '+entity: %d faces, [%s]' % (fn,s))
				has_arm = ob.parent and ob.parent.type == 'ARMATURE'
				arm_name = ob.parent.data.name if has_arm else ''
				current = {
					'node'		: ob.name,
					'material'	: s,
					'mesh'		: '%s@%s' % (ob.data.name, collection_mesh),
					'range'		: [3*offset, 3*(offset+fn)],
					'armature'	: arm_name,
					'actions'	: [],
				}
				entities.append(current)
				offset += fn
		elif ob.type == 'ARMATURE':
			arm = cook_armature(ob.data, log)
			current['node'] = ob.name
			name = ob.data.name
			ani_path = (None, '%s/%s' % (filepath,name))[export_actions]
			anims = save_actions_ext(ani_path, ob, 'pose', log)
			for ani in anims:
				current['actions'].append('%s@%s' % (ani,name))
		elif ob.type == 'CAMERA':
			current = cook_camera(ob.data, log)
			current['node'] = ob.name
			cameras.append(current)
		elif ob.type == 'LAMP':
			current = cook_lamp(ob.data, log)
			current['node'] = ob.name
			lights.append(current)
		# animations
		anims = save_actions_int(out_act_node, ob, None, log)
		for ani in anims:
			current['actions'].append('%s@%s' % (ani, collection_node_anim))
	if out_mesh != None:
		out_mesh.end()
		out_mesh.close()
	if out_act_node != None:
		out_act_node.end()
		out_act_node.close()
	# go!
	document = {
		'global'	: glob,
		'materials'	: materials,
		'nodes'		: nodes,
		'cameras'	: cameras,
		'lights'	: lights,
		'entities'	: entities,
	}
	num_format = '%' + ('.%df' % precision)
	export_json(document, filepath, num_format)
	# finish
	print('Done.')
	log.conclude()
